#!/usr/bin/env python3
"""
Inference script for FunctionGemma CLI Tool Recommender.

This script loads a fine-tuned FunctionGemma model and provides
CLI tool recommendations based on user queries.

Usage:
    python inference.py --model_path ./output/final_model --query "find all python files"
    python inference.py --interactive  # Interactive mode
"""

import argparse
import json
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, List, Any, Optional, Tuple

try:
    from unsloth import FastLanguageModel
    import torch
except ImportError as e:
    print(f"Missing required package: {e}")
    print("Install with: pip install unsloth torch")
    exit(1)


@dataclass
class ToolRecommendation:
    """Represents a single tool recommendation."""
    name: str
    category: str
    confidence: float
    reason: str
    version_hint: Optional[str] = None
    install_cmd: Optional[str] = None
    improvements: Optional[List[str]] = None
    is_alternative: bool = False


@dataclass
class RecommendationResult:
    """Complete recommendation result."""
    primary_tools: List[ToolRecommendation]
    alternative_tools: List[ToolRecommendation]
    task_category: str
    thinking: Optional[str] = None
    raw_response: Optional[str] = None


class SystemContext:
    """Represents the user's system context."""

    VALID_OS = [
        "posix", "linux", "darwin", "ubuntu", "debian",
        "fedora", "arch", "alpine", "bsd", "freebsd",
        "openbsd", "windows"
    ]

    VALID_SHELLS = [
        "sh", "bash", "zsh", "fish", "dash",
        "ksh", "tcsh", "pwsh", "cmd"
    ]

    def __init__(
        self,
        os_type: str = "linux",
        shell: str = "bash",
        prefer_modern_tools: bool = False,
        network_enabled: bool = True,
        installed_tools: Optional[List[str]] = None,
    ):
        self.os_type = os_type.lower() if os_type.lower() in self.VALID_OS else "linux"
        self.shell = shell.lower() if shell.lower() in self.VALID_SHELLS else "bash"
        self.prefer_modern_tools = prefer_modern_tools
        self.network_enabled = network_enabled
        self.installed_tools = installed_tools or []

    def to_prompt_string(self) -> str:
        """Convert context to prompt-friendly string."""
        os_names = {
            "darwin": "darwin (macOS)",
            "ubuntu": "ubuntu (GNU/Linux)",
            "linux": "linux",
            "windows": "windows",
        }
        os_display = os_names.get(self.os_type, self.os_type)

        prefs = []
        if self.prefer_modern_tools:
            prefs.append("prefer modern tools")
        if not self.network_enabled:
            prefs.append("network disabled")

        pref_str = ", ".join(prefs) if prefs else "none specified"

        return f"OS: {os_display}, Shell: {self.shell}, Preferences: {pref_str}"

    @classmethod
    def detect(cls) -> "SystemContext":
        """Auto-detect current system context."""
        import platform
        import os
        import shutil

        # Detect OS
        system = platform.system().lower()
        if system == "darwin":
            os_type = "darwin"
        elif system == "linux":
            # Try to detect distro
            try:
                with open("/etc/os-release") as f:
                    content = f.read().lower()
                    if "ubuntu" in content:
                        os_type = "ubuntu"
                    elif "debian" in content:
                        os_type = "debian"
                    elif "fedora" in content:
                        os_type = "fedora"
                    elif "arch" in content:
                        os_type = "arch"
                    else:
                        os_type = "linux"
            except FileNotFoundError:
                os_type = "linux"
        elif system == "windows":
            os_type = "windows"
        else:
            os_type = "posix"

        # Detect shell
        shell_path = os.environ.get("SHELL", "/bin/bash")
        shell = Path(shell_path).name

        # Check for common modern tools
        modern_tools_installed = []
        for tool in ["rg", "fd", "bat", "fzf", "jq", "htop"]:
            if shutil.which(tool):
                modern_tools_installed.append(tool)

        return cls(
            os_type=os_type,
            shell=shell,
            prefer_modern_tools=len(modern_tools_installed) > 2,
            network_enabled=True,
            installed_tools=modern_tools_installed,
        )


class FunctionGemmaInference:
    """Handles inference with fine-tuned FunctionGemma model."""

    # Special tokens
    DEVELOPER_START = "<start_of_turn>developer\n"
    USER_START = "<start_of_turn>user\n"
    MODEL_START = "<start_of_turn>model\n"
    END_OF_TURN = "<end_of_turn>\n"

    FUNCTION_CALL_START = "<start_function_call>"
    FUNCTION_CALL_END = "<end_function_call>"

    def __init__(
        self,
        model_path: str,
        max_seq_length: int = 4096,
        device: Optional[str] = None,
    ):
        self.model_path = model_path
        self.max_seq_length = max_seq_length
        self.device = device or ("cuda" if torch.cuda.is_available() else "cpu")
        self.model = None
        self.tokenizer = None

    def load_model(self):
        """Load the fine-tuned model."""
        print(f"Loading model from: {self.model_path}")

        self.model, self.tokenizer = FastLanguageModel.from_pretrained(
            model_name=self.model_path,
            max_seq_length=self.max_seq_length,
            load_in_16bit=True,
        )

        # Enable inference mode
        FastLanguageModel.for_inference(self.model)

        print("Model loaded successfully")

    def build_prompt(self, query: str, context: SystemContext) -> str:
        """Build the inference prompt."""
        system_prompt = (
            "You are a CLI tool recommendation assistant. Given the user's query, "
            "OS, shell, and preferences, recommend the most appropriate CLI tools. "
            "Always prefer tools that are likely installed by default, but suggest "
            "modern alternatives when beneficial. Format your response as a function "
            "call to recommend_tools."
        )

        prompt = f"{self.DEVELOPER_START}{system_prompt}{self.END_OF_TURN}"
        prompt += f"{self.USER_START}{context.to_prompt_string()}\nQuery: {query}{self.END_OF_TURN}"
        prompt += self.MODEL_START

        return prompt

    def generate(
        self,
        query: str,
        context: SystemContext,
        max_new_tokens: int = 512,
        temperature: float = 1.0,
        top_k: int = 64,
        top_p: float = 0.95,
    ) -> str:
        """Generate tool recommendations."""
        if self.model is None:
            raise RuntimeError("Model not loaded. Call load_model() first.")

        prompt = self.build_prompt(query, context)

        inputs = self.tokenizer(prompt, return_tensors="pt").to(self.device)

        with torch.no_grad():
            outputs = self.model.generate(
                **inputs,
                max_new_tokens=max_new_tokens,
                temperature=temperature,
                top_k=top_k,
                top_p=top_p,
                do_sample=True,
                pad_token_id=self.tokenizer.eos_token_id,
            )

        response = self.tokenizer.decode(outputs[0], skip_special_tokens=False)

        # Extract only the model's response
        model_response = response.split(self.MODEL_START)[-1]
        if self.END_OF_TURN in model_response:
            model_response = model_response.split(self.END_OF_TURN)[0]

        return model_response

    def parse_response(self, response: str) -> RecommendationResult:
        """Parse the model response into structured recommendations."""
        # Extract thinking block
        thinking = None
        think_match = re.search(r"<think>(.*?)</think>", response, re.DOTALL)
        if think_match:
            thinking = think_match.group(1).strip()

        # Extract function call
        call_match = re.search(
            rf"{re.escape(self.FUNCTION_CALL_START)}call:recommend_tools(.*?){re.escape(self.FUNCTION_CALL_END)}",
            response,
            re.DOTALL
        )

        if not call_match:
            # Try without end tag
            call_match = re.search(
                rf"{re.escape(self.FUNCTION_CALL_START)}call:recommend_tools(.*)",
                response,
                re.DOTALL
            )

        primary_tools = []
        alternative_tools = []
        task_category = "unknown"

        if call_match:
            try:
                json_str = call_match.group(1).strip()
                # Clean up potential JSON issues
                json_str = re.sub(r',\s*}', '}', json_str)
                json_str = re.sub(r',\s*]', ']', json_str)

                data = json.loads(json_str)

                # Parse primary tools
                for tool in data.get("primary_tools", []):
                    primary_tools.append(ToolRecommendation(
                        name=tool.get("name", ""),
                        category=tool.get("category", ""),
                        confidence=tool.get("confidence", 0.5),
                        reason=tool.get("reason", ""),
                        version_hint=tool.get("version_hint"),
                        is_alternative=False,
                    ))

                # Parse alternative tools
                for tool in data.get("alternative_tools", []):
                    alternative_tools.append(ToolRecommendation(
                        name=tool.get("name", ""),
                        category=tool.get("category", ""),
                        confidence=0.0,  # Not installed by default
                        reason=tool.get("reason", ""),
                        install_cmd=tool.get("install_cmd"),
                        improvements=tool.get("improvements"),
                        is_alternative=True,
                    ))

                task_category = data.get("task_category", "unknown")

            except json.JSONDecodeError as e:
                print(f"Warning: Failed to parse JSON response: {e}")

        return RecommendationResult(
            primary_tools=primary_tools,
            alternative_tools=alternative_tools,
            task_category=task_category,
            thinking=thinking,
            raw_response=response,
        )

    def recommend(
        self,
        query: str,
        context: Optional[SystemContext] = None,
    ) -> RecommendationResult:
        """Get tool recommendations for a query."""
        if context is None:
            context = SystemContext.detect()

        response = self.generate(query, context)
        return self.parse_response(response)


def format_recommendations(result: RecommendationResult, verbose: bool = False) -> str:
    """Format recommendations for display."""
    output = []

    output.append(f"\n{'=' * 60}")
    output.append(f"Task Category: {result.task_category}")
    output.append('=' * 60)

    if verbose and result.thinking:
        output.append("\nReasoning:")
        output.append("-" * 40)
        output.append(result.thinking)

    if result.primary_tools:
        output.append("\nPrimary Tools (likely installed):")
        output.append("-" * 40)
        for tool in result.primary_tools:
            conf_pct = int(tool.confidence * 100)
            output.append(f"  {tool.name} [{tool.category}] - {conf_pct}% likely installed")
            output.append(f"    {tool.reason}")
            if tool.version_hint:
                output.append(f"    Version: {tool.version_hint}")

    if result.alternative_tools:
        output.append("\nAlternative Tools (may require installation):")
        output.append("-" * 40)
        for tool in result.alternative_tools:
            output.append(f"  {tool.name} [{tool.category}]")
            output.append(f"    {tool.reason}")
            if tool.install_cmd:
                output.append(f"    Install: {tool.install_cmd}")
            if tool.improvements:
                output.append(f"    Improvements: {', '.join(tool.improvements)}")

    output.append("")

    return "\n".join(output)


def interactive_mode(inference: FunctionGemmaInference, context: SystemContext, verbose: bool):
    """Run in interactive mode."""
    print("\n" + "=" * 60)
    print("CLI Tool Recommender - Interactive Mode")
    print("=" * 60)
    print(f"System: {context.to_prompt_string()}")
    print("Type 'quit' or 'exit' to exit")
    print("Type 'context' to change system context")
    print("=" * 60)

    while True:
        try:
            query = input("\nQuery: ").strip()
        except (KeyboardInterrupt, EOFError):
            print("\nGoodbye!")
            break

        if not query:
            continue

        if query.lower() in ("quit", "exit", "q"):
            print("Goodbye!")
            break

        if query.lower() == "context":
            print(f"\nCurrent: {context.to_prompt_string()}")
            new_os = input("New OS (or press Enter to keep): ").strip()
            if new_os:
                context.os_type = new_os
            new_shell = input("New Shell (or press Enter to keep): ").strip()
            if new_shell:
                context.shell = new_shell
            print(f"Updated: {context.to_prompt_string()}")
            continue

        try:
            result = inference.recommend(query, context)
            print(format_recommendations(result, verbose=verbose))
        except Exception as e:
            print(f"Error: {e}")


def main():
    parser = argparse.ArgumentParser(
        description="CLI Tool Recommender using fine-tuned FunctionGemma"
    )
    parser.add_argument(
        "--model_path",
        type=str,
        default="./output/final_model",
        help="Path to fine-tuned model"
    )
    parser.add_argument(
        "--query",
        type=str,
        default=None,
        help="Query for tool recommendation"
    )
    parser.add_argument(
        "--os",
        type=str,
        default=None,
        help="Operating system (darwin, ubuntu, linux, windows, etc.)"
    )
    parser.add_argument(
        "--shell",
        type=str,
        default=None,
        help="Shell type (bash, zsh, fish, etc.)"
    )
    parser.add_argument(
        "--modern",
        action="store_true",
        help="Prefer modern tools"
    )
    parser.add_argument(
        "--no-network",
        action="store_true",
        help="Disable network (no install suggestions)"
    )
    parser.add_argument(
        "--interactive",
        action="store_true",
        help="Run in interactive mode"
    )
    parser.add_argument(
        "--verbose",
        "-v",
        action="store_true",
        help="Show reasoning and raw response"
    )
    parser.add_argument(
        "--detect",
        action="store_true",
        help="Auto-detect system context"
    )

    args = parser.parse_args()

    # Set up context
    if args.detect:
        context = SystemContext.detect()
        print(f"Detected: {context.to_prompt_string()}")
    else:
        context = SystemContext(
            os_type=args.os or "linux",
            shell=args.shell or "bash",
            prefer_modern_tools=args.modern,
            network_enabled=not args.no_network,
        )

    # Load model
    inference = FunctionGemmaInference(model_path=args.model_path)
    inference.load_model()

    if args.interactive or args.query is None:
        interactive_mode(inference, context, args.verbose)
    else:
        result = inference.recommend(args.query, context)
        print(format_recommendations(result, verbose=args.verbose))


if __name__ == "__main__":
    main()
