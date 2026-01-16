#!/usr/bin/env python3
"""
Fine-tuning script for FunctionGemma CLI Tool Recommender.

Based on official Unsloth FunctionGemma notebook with key improvements:
- Uses tokenizer.apply_chat_template() for proper formatting
- train_on_responses_only() to only train on assistant outputs
- HF-style tool schemas for proper tool declaration
- Higher LoRA rank (r=128) for better quality
- SFTConfig instead of TrainingArguments

Usage:
    python finetune.py --data_path ./data/training_data.json --output_dir ./output
"""

import argparse
import json
import re
from pathlib import Path
from typing import Dict, List, Any, Optional, Tuple

# Check for required packages
try:
    from unsloth import FastLanguageModel
    from unsloth.chat_templates import train_on_responses_only
    import torch
    from datasets import Dataset
    from trl import SFTTrainer, SFTConfig
except ImportError as e:
    print(f"Missing required package: {e}")
    print("Install with: uv pip install unsloth")
    exit(1)


# Model configuration
MODEL_CONFIG = {
    "model_name": "unsloth/functiongemma-270m-it",
    "max_seq_length": 4096,
    "load_in_4bit": False,
    "load_in_8bit": False,
    "load_in_16bit": True,
    "full_finetuning": False,
}

# LoRA configuration - higher rank for better quality (from official notebook)
LORA_CONFIG = {
    "r": 128,  # Higher rank for better quality (was 16)
    "lora_alpha": 256,  # 2x rank as recommended
    "lora_dropout": 0,
    "target_modules": [
        "q_proj", "k_proj", "v_proj", "o_proj",
        "gate_proj", "up_proj", "down_proj"
    ],
    "bias": "none",
    "use_gradient_checkpointing": "unsloth",  # 30% less VRAM
    "random_state": 3407,
    "use_rslora": False,
    "loftq_config": None,
}

# Training configuration
TRAINING_CONFIG = {
    "per_device_train_batch_size": 4,
    "gradient_accumulation_steps": 2,
    "warmup_steps": 10,
    "max_steps": 500,  # Set to None and use num_train_epochs for full run
    "learning_rate": 2e-4,
    "logging_steps": 1,
    "optim": "adamw_8bit",
    "weight_decay": 0.001,
    "lr_scheduler_type": "linear",
    "seed": 3407,
    "report_to": "none",
}


# CLI Tool Recommendation function schema (HF-style)
CLI_TOOL_SCHEMA = {
    "type": "function",
    "function": {
        "name": "recommend_tools",
        "description": "Recommend CLI tools for a given task based on user's OS, shell, and preferences. Returns primary tools (likely installed) and alternative tools (may require installation).",
        "parameters": {
            "type": "object",
            "properties": {
                "primary_tools": {
                    "type": "array",
                    "description": "Tools most likely installed by default on the system",
                    "items": {
                        "type": "object",
                        "properties": {
                            "name": {"type": "string", "description": "Tool name (e.g., 'find', 'grep')"},
                            "category": {"type": "string", "description": "Tool category"},
                            "confidence": {"type": "number", "description": "0.0-1.0 confidence tool is installed"},
                            "reason": {"type": "string", "description": "Why this tool is recommended"},
                        },
                        "required": ["name", "category", "confidence", "reason"]
                    }
                },
                "alternative_tools": {
                    "type": "array",
                    "description": "Modern alternatives that may need installation",
                    "items": {
                        "type": "object",
                        "properties": {
                            "name": {"type": "string", "description": "Tool name"},
                            "category": {"type": "string", "description": "Tool category"},
                            "install_cmd": {"type": "string", "description": "Installation command"},
                            "reason": {"type": "string", "description": "Why this is recommended"},
                            "improvements": {"type": "array", "items": {"type": "string"}},
                        },
                        "required": ["name", "install_cmd", "reason"]
                    }
                },
                "task_category": {
                    "type": "string",
                    "description": "Category of the user's task"
                }
            },
            "required": ["primary_tools", "task_category"]
        }
    }
}

THINK_TAG_OPEN = "<think>"
THINK_TAG_CLOSE = "</think>"


def prepare_messages_and_tools(example: Dict[str, Any]) -> Tuple[Optional[List[Dict]], List[Dict]]:
    """
    Prepare messages and tools from training example.
    Converts our format to HF-style format for apply_chat_template.

    This handles:
    - System/developer messages
    - User messages
    - Assistant messages with thinking and tool calls
    """
    context = example.get("context", {})
    query = example.get("user_query", "")
    conversation = example.get("conversation", [])

    if not conversation:
        return None, []

    messages = []
    tools = [CLI_TOOL_SCHEMA]

    for turn in conversation:
        role = turn.get("role", "")
        content = turn.get("content", "")

        if role == "developer":
            # System message
            messages.append({
                "role": "system",
                "content": content
            })

        elif role == "user":
            messages.append({
                "role": "user",
                "content": content
            })

        elif role == "model":
            # Parse model response for thinking and tool calls
            assistant_msg = {"role": "assistant"}

            # Extract thinking block
            thinking_match = re.search(
                rf"{THINK_TAG_OPEN}(.*?){THINK_TAG_CLOSE}",
                content,
                re.DOTALL
            )

            # Check for function call in content
            func_match = re.search(
                r"<start_function_call>call:recommend_tools(.*?)<end_function_call>",
                content,
                re.DOTALL
            )

            if func_match:
                try:
                    args_json = func_match.group(1).strip()
                    # Clean up JSON
                    args_json = re.sub(r',\s*}', '}', args_json)
                    args_json = re.sub(r',\s*]', ']', args_json)
                    args = json.loads(args_json)

                    # Build assistant message with thinking + tool call
                    if thinking_match:
                        thinking_text = thinking_match.group(1).strip()
                        assistant_msg["content"] = f"{THINK_TAG_OPEN}{thinking_text}{THINK_TAG_CLOSE}"
                    else:
                        assistant_msg["content"] = ""

                    assistant_msg["tool_calls"] = [{
                        "id": "call_1",
                        "type": "function",
                        "function": {
                            "name": "recommend_tools",
                            "arguments": args
                        }
                    }]

                except json.JSONDecodeError as e:
                    # Fallback: use content as-is
                    assistant_msg["content"] = content
            else:
                # No function call, use content as-is
                assistant_msg["content"] = content

            messages.append(assistant_msg)

    # Validate: must have thinking in assistant messages
    has_thinking = False
    for msg in messages:
        if msg.get("role") == "assistant":
            content = msg.get("content", "")
            if THINK_TAG_OPEN in content and THINK_TAG_CLOSE in content:
                has_thinking = True
                break

    if not has_thinking:
        return None, []

    return messages, tools


def format_example(example: Dict[str, Any], tokenizer) -> Optional[str]:
    """
    Format a single training example using tokenizer.apply_chat_template().
    This is the proper way to format FunctionGemma prompts.
    """
    messages, tools = prepare_messages_and_tools(example)

    if messages is None or len(messages) == 0:
        return None

    try:
        chat_str = tokenizer.apply_chat_template(
            messages,
            tools=tools,
            add_generation_prompt=False,
            tokenize=False,
        )
        # Remove BOS token if present
        if chat_str.startswith("<bos>"):
            chat_str = chat_str[5:]
        return chat_str
    except Exception as e:
        print(f"Warning: Failed to apply chat template: {e}")
        return None


def load_training_data(data_path: str) -> List[Dict[str, Any]]:
    """Load training data from JSON file."""
    with open(data_path, 'r') as f:
        data = json.load(f)

    if isinstance(data, dict) and "examples" in data:
        return data["examples"]
    return data


def create_dataset(examples: List[Dict[str, Any]], tokenizer) -> Dataset:
    """Create a HuggingFace Dataset with properly formatted texts."""
    formatted = []

    for i, example in enumerate(examples):
        text = format_example(example, tokenizer)
        if text is not None:
            formatted.append({"text": text})
        if (i + 1) % 100 == 0:
            print(f"  Formatted {i + 1}/{len(examples)} examples...")

    print(f"Successfully formatted {len(formatted)}/{len(examples)} examples")

    if not formatted:
        raise ValueError("No valid training examples after formatting!")

    return Dataset.from_list(formatted)


def load_model_and_tokenizer():
    """Load the FunctionGemma model and tokenizer."""
    print(f"Loading model: {MODEL_CONFIG['model_name']}")

    model, tokenizer = FastLanguageModel.from_pretrained(
        model_name=MODEL_CONFIG["model_name"],
        max_seq_length=MODEL_CONFIG["max_seq_length"],
        load_in_4bit=MODEL_CONFIG["load_in_4bit"],
        load_in_8bit=MODEL_CONFIG["load_in_8bit"],
        load_in_16bit=MODEL_CONFIG["load_in_16bit"],
        full_finetuning=MODEL_CONFIG["full_finetuning"],
    )

    return model, tokenizer


def apply_lora(model):
    """Apply LoRA adapters to the model."""
    print(f"Applying LoRA adapters (r={LORA_CONFIG['r']}, alpha={LORA_CONFIG['lora_alpha']})...")

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
        loftq_config=LORA_CONFIG["loftq_config"],
    )

    return model


def create_trainer(model, tokenizer, dataset, output_dir: str) -> SFTTrainer:
    """Create the SFT trainer with train_on_responses_only."""

    trainer = SFTTrainer(
        model=model,
        tokenizer=tokenizer,
        train_dataset=dataset,
        eval_dataset=None,
        args=SFTConfig(
            dataset_text_field="text",
            per_device_train_batch_size=TRAINING_CONFIG["per_device_train_batch_size"],
            gradient_accumulation_steps=TRAINING_CONFIG["gradient_accumulation_steps"],
            warmup_steps=TRAINING_CONFIG["warmup_steps"],
            max_steps=TRAINING_CONFIG["max_steps"],
            learning_rate=TRAINING_CONFIG["learning_rate"],
            logging_steps=TRAINING_CONFIG["logging_steps"],
            optim=TRAINING_CONFIG["optim"],
            weight_decay=TRAINING_CONFIG["weight_decay"],
            lr_scheduler_type=TRAINING_CONFIG["lr_scheduler_type"],
            seed=TRAINING_CONFIG["seed"],
            output_dir=output_dir,
            report_to=TRAINING_CONFIG["report_to"],
        ),
    )

    # IMPORTANT: Only train on assistant responses, not user inputs
    # This significantly improves finetuning quality
    print("Applying train_on_responses_only (masking user inputs)...")
    trainer = train_on_responses_only(
        trainer,
        instruction_part="<start_of_turn>user\n",
        response_part="<start_of_turn>model\n",
    )

    return trainer


def save_model(model, tokenizer, output_dir: str, save_method: str = "lora"):
    """Save the fine-tuned model."""
    output_path = Path(output_dir) / "final_model"
    output_path.mkdir(parents=True, exist_ok=True)

    print(f"Saving model to {output_path} (method: {save_method})")

    if save_method == "lora":
        model.save_pretrained(str(output_path))
        tokenizer.save_pretrained(str(output_path))
        print("Saved LoRA adapters")

    elif save_method == "merged_16bit":
        model.save_pretrained_merged(
            str(output_path),
            tokenizer,
            save_method="merged_16bit"
        )
        print("Saved merged 16-bit model")

    elif save_method == "merged_4bit":
        model.save_pretrained_merged(
            str(output_path),
            tokenizer,
            save_method="merged_4bit"
        )
        print("Saved merged 4-bit model")

    elif save_method == "gguf":
        model.save_pretrained_gguf(
            str(output_path),
            tokenizer,
            quantization_method="Q8_0"  # Q8_0, BF16, F16 supported
        )
        print("Saved GGUF quantized model (Q8_0)")

    return str(output_path)


def show_memory_stats():
    """Show GPU memory statistics."""
    if torch.cuda.is_available():
        gpu_stats = torch.cuda.get_device_properties(0)
        reserved = round(torch.cuda.max_memory_reserved() / 1024 / 1024 / 1024, 3)
        total = round(gpu_stats.total_memory / 1024 / 1024 / 1024, 3)
        print(f"GPU: {gpu_stats.name}")
        print(f"Memory: {reserved} GB / {total} GB ({round(reserved/total*100, 1)}%)")
    else:
        print("No GPU available")


def main():
    parser = argparse.ArgumentParser(
        description="Fine-tune FunctionGemma for CLI tool recommendation"
    )
    parser.add_argument(
        "--data_path",
        type=str,
        default="./data/training_examples.json",
        help="Path to training data JSON file"
    )
    parser.add_argument(
        "--output_dir",
        type=str,
        default="./output",
        help="Output directory for model and logs"
    )
    parser.add_argument(
        "--max_steps",
        type=int,
        default=500,
        help="Max training steps (set to -1 for full epochs)"
    )
    parser.add_argument(
        "--batch_size",
        type=int,
        default=4,
        help="Training batch size"
    )
    parser.add_argument(
        "--learning_rate",
        type=float,
        default=2e-4,
        help="Learning rate"
    )
    parser.add_argument(
        "--lora_r",
        type=int,
        default=128,
        help="LoRA rank (higher = better quality, more memory)"
    )
    parser.add_argument(
        "--save_method",
        type=str,
        choices=["lora", "merged_16bit", "merged_4bit", "gguf"],
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
    TRAINING_CONFIG["max_steps"] = args.max_steps if args.max_steps > 0 else None
    TRAINING_CONFIG["per_device_train_batch_size"] = args.batch_size
    TRAINING_CONFIG["learning_rate"] = args.learning_rate
    LORA_CONFIG["r"] = args.lora_r
    LORA_CONFIG["lora_alpha"] = args.lora_r * 2

    # Create output directory
    output_dir = Path(args.output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    print("=" * 60)
    print("FunctionGemma CLI Tool Recommender Fine-tuning")
    print("=" * 60)
    print(f"Model: {MODEL_CONFIG['model_name']}")
    print(f"LoRA: r={LORA_CONFIG['r']}, alpha={LORA_CONFIG['lora_alpha']}")
    print(f"Batch size: {TRAINING_CONFIG['per_device_train_batch_size']}")
    print(f"Learning rate: {TRAINING_CONFIG['learning_rate']}")
    print(f"Max steps: {TRAINING_CONFIG['max_steps']}")
    print()

    # Load model and tokenizer
    print("Loading model and tokenizer...")
    model, tokenizer = load_model_and_tokenizer()

    # Apply LoRA
    model = apply_lora(model)

    # Load and format training data
    print(f"\nLoading training data from: {args.data_path}")
    training_examples = load_training_data(args.data_path)
    print(f"Loaded {len(training_examples)} raw examples")

    # Create dataset with proper formatting
    print("\nFormatting data using apply_chat_template...")
    dataset = create_dataset(training_examples, tokenizer)
    print(f"Dataset ready: {len(dataset)} examples")

    # Show sample
    print("\nSample formatted text (first 500 chars):")
    print("-" * 40)
    print(dataset[0]["text"][:500] + "...")

    # Create trainer
    print("\nInitializing trainer...")
    trainer = create_trainer(model, tokenizer, dataset, str(output_dir))

    # Show initial memory
    print("\nInitial GPU memory:")
    show_memory_stats()

    # Train
    print("\n" + "=" * 60)
    print("Starting training...")
    print("=" * 60)

    trainer_stats = trainer.train(resume_from_checkpoint=args.resume_from)

    # Show final stats
    print("\n" + "=" * 60)
    print("Training complete!")
    print("=" * 60)
    runtime = trainer_stats.metrics.get('train_runtime', 0)
    print(f"Training time: {runtime:.1f}s ({runtime/60:.1f} minutes)")
    show_memory_stats()

    # Save model
    print("\nSaving model...")
    model_path = save_model(model, tokenizer, str(output_dir), args.save_method)

    print("\n" + "=" * 60)
    print(f"Model saved to: {model_path}")
    print("=" * 60)


if __name__ == "__main__":
    main()
