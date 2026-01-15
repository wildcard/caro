#!/usr/bin/env python3
"""
Fine-tuning script for FunctionGemma CLI Tool Recommender.

This script fine-tunes FunctionGemma to recommend CLI tools based on:
- User query/prompt
- Operating system (POSIX, Linux, macOS, Ubuntu, BSD, Windows)
- Shell type (sh, bash, zsh, fish, etc.)
- User preferences and configuration

Usage:
    python finetune.py --data_path ./data/training_data.json --output_dir ./output
"""

import argparse
import json
import os
from pathlib import Path
from typing import Dict, List, Any, Optional

# Check for required packages
try:
    from unsloth import FastLanguageModel
    from unsloth.chat_templates import get_chat_template
    import torch
    from datasets import Dataset
    from trl import SFTTrainer
    from transformers import TrainingArguments
except ImportError as e:
    print(f"Missing required package: {e}")
    print("Install with: pip install unsloth datasets trl transformers torch")
    exit(1)


# Model configuration
MODEL_CONFIG = {
    "model_name": "unsloth/functiongemma-270m-it",
    "max_seq_length": 4096,
    "load_in_16bit": True,
    "full_finetuning": False,
}

# LoRA configuration for efficient fine-tuning
LORA_CONFIG = {
    "r": 16,  # Rank of LoRA
    "lora_alpha": 16,
    "lora_dropout": 0,
    "target_modules": [
        "q_proj", "k_proj", "v_proj", "o_proj",
        "gate_proj", "up_proj", "down_proj"
    ],
    "bias": "none",
    "use_gradient_checkpointing": "unsloth",
    "random_state": 42,
    "use_rslora": False,
}

# Training configuration
TRAINING_CONFIG = {
    "per_device_train_batch_size": 2,
    "gradient_accumulation_steps": 4,
    "warmup_steps": 5,
    "num_train_epochs": 3,
    "learning_rate": 2e-4,
    "fp16": not torch.cuda.is_bf16_supported() if torch.cuda.is_available() else True,
    "bf16": torch.cuda.is_bf16_supported() if torch.cuda.is_available() else False,
    "logging_steps": 1,
    "optim": "adamw_8bit",
    "weight_decay": 0.01,
    "lr_scheduler_type": "linear",
    "seed": 42,
}


class FunctionGemmaChatFormatter:
    """Format training data for FunctionGemma chat template."""

    # FunctionGemma special tokens
    DEVELOPER_START = "<start_of_turn>developer\n"
    USER_START = "<start_of_turn>user\n"
    MODEL_START = "<start_of_turn>model\n"
    END_OF_TURN = "<end_of_turn>\n"

    FUNCTION_DECLARATION_START = "<start_function_declaration>"
    FUNCTION_CALL_START = "<start_function_call>"
    FUNCTION_CALL_END = "<end_function_call>"
    FUNCTION_RESPONSE_START = "<start_function_response>"

    def __init__(self, function_schemas: Optional[Dict[str, Any]] = None):
        self.function_schemas = function_schemas or self._load_default_schemas()

    def _load_default_schemas(self) -> Dict[str, Any]:
        """Load default function schemas."""
        schema_path = Path(__file__).parent.parent / "schemas" / "tool_functions.py"
        if schema_path.exists():
            import importlib.util
            spec = importlib.util.spec_from_file_location("tool_functions", schema_path)
            module = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(module)
            return module.FUNCTION_SCHEMAS
        return {}

    def format_function_declarations(self) -> str:
        """Format all function declarations for the developer message."""
        declarations = []
        for name, schema in self.function_schemas.items():
            decl = f"{self.FUNCTION_DECLARATION_START}declaration:{name}"
            decl += json.dumps(schema, indent=2)
            declarations.append(decl)
        return "\n".join(declarations)

    def format_conversation(self, example: Dict[str, Any]) -> str:
        """Format a single training example into FunctionGemma chat format."""
        formatted = ""

        # Build the conversation
        for turn in example.get("conversation", []):
            role = turn.get("role", "")
            content = turn.get("content", "")

            if role == "developer":
                formatted += self.DEVELOPER_START
                formatted += content
                if self.function_schemas:
                    formatted += "\n\n" + self.format_function_declarations()
                formatted += self.END_OF_TURN

            elif role == "user":
                formatted += self.USER_START
                formatted += content
                formatted += self.END_OF_TURN

            elif role == "model":
                formatted += self.MODEL_START
                formatted += content
                formatted += self.END_OF_TURN

        return formatted

    def format_dataset(self, examples: List[Dict[str, Any]]) -> List[str]:
        """Format all training examples."""
        return [self.format_conversation(ex) for ex in examples]


def load_training_data(data_path: str) -> List[Dict[str, Any]]:
    """Load training data from JSON file."""
    with open(data_path, 'r') as f:
        data = json.load(f)

    # Handle both raw list and structured format
    if isinstance(data, dict) and "examples" in data:
        return data["examples"]
    return data


def create_dataset(formatted_texts: List[str]) -> Dataset:
    """Create a HuggingFace Dataset from formatted texts."""
    return Dataset.from_dict({"text": formatted_texts})


def load_model_and_tokenizer():
    """Load the FunctionGemma model and tokenizer."""
    print(f"Loading model: {MODEL_CONFIG['model_name']}")

    model, tokenizer = FastLanguageModel.from_pretrained(
        model_name=MODEL_CONFIG["model_name"],
        max_seq_length=MODEL_CONFIG["max_seq_length"],
        load_in_16bit=MODEL_CONFIG["load_in_16bit"],
        full_finetuning=MODEL_CONFIG["full_finetuning"],
    )

    return model, tokenizer


def apply_lora(model):
    """Apply LoRA adapters to the model."""
    print("Applying LoRA adapters...")

    model = FastLanguageModel.get_peft_model(
        model,
        r=LORA_CONFIG["r"],
        lora_alpha=LORA_CONFIG["lora_alpha"],
        lora_dropout=LORA_CONFIG["lora_dropout"],
        target_modules=LORA_CONFIG["target_modules"],
        bias=LORA_CONFIG["bias"],
        use_gradient_checkpointing=LORA_CONFIG["use_gradient_checkpointing"],
        random_state=LORA_CONFIG["random_state"],
        use_rslora=LORA_CONFIG["use_rslora"],
    )

    return model


def create_trainer(model, tokenizer, dataset, output_dir: str) -> SFTTrainer:
    """Create the SFT trainer."""
    training_args = TrainingArguments(
        output_dir=output_dir,
        per_device_train_batch_size=TRAINING_CONFIG["per_device_train_batch_size"],
        gradient_accumulation_steps=TRAINING_CONFIG["gradient_accumulation_steps"],
        warmup_steps=TRAINING_CONFIG["warmup_steps"],
        num_train_epochs=TRAINING_CONFIG["num_train_epochs"],
        learning_rate=TRAINING_CONFIG["learning_rate"],
        fp16=TRAINING_CONFIG["fp16"],
        bf16=TRAINING_CONFIG["bf16"],
        logging_steps=TRAINING_CONFIG["logging_steps"],
        optim=TRAINING_CONFIG["optim"],
        weight_decay=TRAINING_CONFIG["weight_decay"],
        lr_scheduler_type=TRAINING_CONFIG["lr_scheduler_type"],
        seed=TRAINING_CONFIG["seed"],
        report_to="none",  # Disable wandb/tensorboard for simplicity
    )

    trainer = SFTTrainer(
        model=model,
        tokenizer=tokenizer,
        train_dataset=dataset,
        dataset_text_field="text",
        max_seq_length=MODEL_CONFIG["max_seq_length"],
        args=training_args,
    )

    return trainer


def save_model(model, tokenizer, output_dir: str, save_method: str = "lora"):
    """Save the fine-tuned model."""
    output_path = Path(output_dir) / "final_model"
    output_path.mkdir(parents=True, exist_ok=True)

    print(f"Saving model to {output_path}")

    if save_method == "lora":
        # Save only LoRA adapters (small file size)
        model.save_pretrained(str(output_path))
        tokenizer.save_pretrained(str(output_path))
        print("Saved LoRA adapters")

    elif save_method == "merged_16bit":
        # Merge and save in 16-bit
        model.save_pretrained_merged(
            str(output_path),
            tokenizer,
            save_method="merged_16bit"
        )
        print("Saved merged 16-bit model")

    elif save_method == "gguf":
        # Save in GGUF format for llama.cpp compatibility
        model.save_pretrained_gguf(
            str(output_path),
            tokenizer,
            quantization_method="q4_k_m"
        )
        print("Saved GGUF quantized model")

    return str(output_path)


def main():
    parser = argparse.ArgumentParser(
        description="Fine-tune FunctionGemma for CLI tool recommendation"
    )
    parser.add_argument(
        "--data_path",
        type=str,
        default="./data/training_data.json",
        help="Path to training data JSON file"
    )
    parser.add_argument(
        "--output_dir",
        type=str,
        default="./output",
        help="Output directory for model and logs"
    )
    parser.add_argument(
        "--epochs",
        type=int,
        default=3,
        help="Number of training epochs"
    )
    parser.add_argument(
        "--batch_size",
        type=int,
        default=2,
        help="Training batch size"
    )
    parser.add_argument(
        "--learning_rate",
        type=float,
        default=2e-4,
        help="Learning rate"
    )
    parser.add_argument(
        "--save_method",
        type=str,
        choices=["lora", "merged_16bit", "gguf"],
        default="lora",
        help="How to save the model"
    )
    parser.add_argument(
        "--resume_from",
        type=str,
        default=None,
        help="Resume training from checkpoint"
    )

    args = parser.parse_args()

    # Update config with CLI args
    TRAINING_CONFIG["num_train_epochs"] = args.epochs
    TRAINING_CONFIG["per_device_train_batch_size"] = args.batch_size
    TRAINING_CONFIG["learning_rate"] = args.learning_rate

    # Create output directory
    output_dir = Path(args.output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    print("=" * 60)
    print("FunctionGemma CLI Tool Recommender Fine-tuning")
    print("=" * 60)

    # Load training data
    print(f"\nLoading training data from: {args.data_path}")
    training_examples = load_training_data(args.data_path)
    print(f"Loaded {len(training_examples)} training examples")

    # Format data for FunctionGemma
    print("\nFormatting data for FunctionGemma chat template...")
    formatter = FunctionGemmaChatFormatter()
    formatted_texts = formatter.format_dataset(training_examples)

    # Create dataset
    dataset = create_dataset(formatted_texts)
    print(f"Created dataset with {len(dataset)} examples")

    # Load model and tokenizer
    print("\nLoading model and tokenizer...")
    model, tokenizer = load_model_and_tokenizer()

    # Apply LoRA
    model = apply_lora(model)

    # Create trainer
    print("\nInitializing trainer...")
    trainer = create_trainer(model, tokenizer, dataset, str(output_dir))

    # Resume from checkpoint if specified
    if args.resume_from:
        print(f"Resuming from checkpoint: {args.resume_from}")

    # Train
    print("\n" + "=" * 60)
    print("Starting training...")
    print("=" * 60)

    trainer.train(resume_from_checkpoint=args.resume_from)

    # Save model
    print("\n" + "=" * 60)
    print("Saving model...")
    print("=" * 60)

    model_path = save_model(model, tokenizer, str(output_dir), args.save_method)

    print("\n" + "=" * 60)
    print("Training complete!")
    print(f"Model saved to: {model_path}")
    print("=" * 60)


if __name__ == "__main__":
    main()
