"""
Harbor agent wrapper for cmdai - converts Terminal-Bench tasks into shell commands.

This agent wraps the cmdai CLI tool to work with Harbor's Terminal-Bench 2.0.
"""

import asyncio
import json
import subprocess
from pathlib import Path
from typing import Optional

from harbor.agents.base import BaseAgent
from harbor.environments.base import BaseEnvironment
from harbor.models.agent.context import AgentContext
from harbor.models.agent.name import AgentName


class CmdaiAgent(BaseAgent):
    """
    Harbor agent that uses cmdai for command generation.

    cmdai is a single-shot command generator, so this agent implements
    a multi-turn loop to complete complex tasks.
    """

    def __init__(
        self,
        logs_dir: Path,
        model_name: Optional[str] = None,
        max_turns: int = 50,
        *args,
        **kwargs,
    ):
        """Initialize cmdai agent.

        Args:
            logs_dir: Directory to store logs
            model_name: Model name (for compatibility, cmdai uses embedded models)
            max_turns: Maximum number of command execution turns
            *args: Additional arguments
            **kwargs: Additional keyword arguments
        """
        super().__init__(logs_dir, model_name, *args, **kwargs)
        self._max_turns = max_turns
        self._cmdai_binary = "/usr/local/bin/cmdai"

    @staticmethod
    def name() -> str:
        return "cmdai"

    def version(self) -> Optional[str]:
        return "0.1.0"

    async def setup(self, environment: BaseEnvironment) -> None:
        """Install cmdai in the container environment."""
        # Create installation directory
        await environment.exec(command="mkdir -p /usr/local/bin")

        # Upload the cmdai binary
        cmdai_binary_path = Path("/home/user/cmdai/target/release/cmdai")
        if not cmdai_binary_path.exists():
            raise FileNotFoundError(f"cmdai binary not found at {cmdai_binary_path}")

        await environment.upload_file(
            source_path=cmdai_binary_path,
            target_path=self._cmdai_binary,
        )

        # Make it executable
        await environment.exec(command=f"chmod +x {self._cmdai_binary}")

        # Verify installation
        result = await environment.exec(command=f"{self._cmdai_binary} --version")
        setup_dir = self.logs_dir / "setup"
        setup_dir.mkdir(parents=True, exist_ok=True)

        (setup_dir / "version.txt").write_text(result.stdout or "")
        if result.return_code != 0:
            raise RuntimeError(f"Failed to install cmdai: {result.stderr}")

    async def run(
        self,
        instruction: str,
        environment: BaseEnvironment,
        context: AgentContext,
    ) -> None:
        """
        Execute the task using cmdai's command generation.

        This implements a simple multi-turn loop:
        1. Generate command using cmdai
        2. Execute the command
        3. Observe the result
        4. Repeat until task appears complete or max turns reached
        """
        # Initialize context
        context.n_input_tokens = 0
        context.n_output_tokens = 0
        context.metadata = {"turns": 0}

        current_prompt = instruction
        turn = 0

        for turn in range(self._max_turns):
            turn_dir = self.logs_dir / f"turn-{turn}"
            turn_dir.mkdir(parents=True, exist_ok=True)

            # Save the prompt
            (turn_dir / "prompt.txt").write_text(current_prompt)

            # Generate command using cmdai
            # Use --output json to get structured response
            cmdai_command = f'{self._cmdai_binary} --output json "{current_prompt}"'

            result = await environment.exec(
                command=cmdai_command,
                timeout_sec=30,
            )

            (turn_dir / "cmdai_stdout.txt").write_text(result.stdout or "")
            (turn_dir / "cmdai_stderr.txt").write_text(result.stderr or "")
            (turn_dir / "cmdai_return_code.txt").write_text(str(result.return_code))

            if result.return_code != 0:
                # cmdai failed, try to continue with a simple approach
                current_prompt = f"Previous command generation failed. {instruction}"
                continue

            # Parse JSON output
            try:
                response = json.loads(result.stdout or "{}")
                generated_command = response.get("command", "")

                if not generated_command:
                    break

                (turn_dir / "generated_command.txt").write_text(generated_command)

                # Execute the generated command
                exec_result = await environment.exec(
                    command=generated_command,
                    timeout_sec=60,
                )

                (turn_dir / "exec_stdout.txt").write_text(exec_result.stdout or "")
                (turn_dir / "exec_stderr.txt").write_text(exec_result.stderr or "")
                (turn_dir / "exec_return_code.txt").write_text(str(exec_result.return_code))

                # Check if task might be complete
                # Simple heuristic: if command succeeded and output suggests completion
                output = (exec_result.stdout or "") + (exec_result.stderr or "")

                # Update prompt for next iteration
                if exec_result.return_code == 0:
                    current_prompt = f"{instruction}\n\nPrevious action completed successfully:\n{generated_command}\nOutput: {output[:500]}\n\nWhat's the next step?"
                else:
                    current_prompt = f"{instruction}\n\nPrevious action failed:\n{generated_command}\nError: {output[:500]}\n\nHow should we fix this?"

            except json.JSONDecodeError:
                # Couldn't parse output, continue
                break

        context.metadata["turns"] = turn + 1
