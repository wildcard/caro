export const useCases = [
  {
    icon: "\uD83D\uDEF0\uFE0F",
    title: "On-Call SRE",
    problem: "At 2 AM during an incident, you can't afford to mistype a command.",
    solution: "Describe what you need in plain English. Caro generates the command and catches dangerous patterns before you make a career-ending mistake.",
    example: {
      input: "find and kill the runaway process eating CPU",
      output: "ps aux | sort -nrk 3,3 | head -1 | awk '{print $2}' | xargs kill"
    }
  },
  {
    icon: "\uD83D\uDD12",
    title: "Security-Conscious Engineer",
    problem: "You can't send your production server names and file paths to OpenAI.",
    solution: "Caro runs 100% locally. Your commands, configs, and infrastructure details never leave your machine. Zero telemetry, auditable code.",
    example: {
      input: "search logs for failed auth from IP 192.168.1.100",
      output: "grep -r 'failed.*auth.*192\.168\.1\.100' /var/log/"
    }
  },
  {
    icon: "\uD83C\uDF0D",
    title: "Cross-Platform Developer",
    problem: "Your commands work on Mac but break on the Linux CI server.",
    solution: "Caro detects your platform and generates commands that work. BSD vs GNU, sed vs gsed, find flags—all handled automatically.",
    example: {
      input: "replace all tabs with spaces in python files",
      output: "find . -name '*.py' -exec sed -i '' 's/\t/    /g' {} +"
    }
  },
  {
    icon: "\uD83D\uDE80",
    title: "Terminal Power User",
    problem: "You waste time looking up complex command syntax on Stack Overflow.",
    solution: "Describe your intent. Get the exact command in under 2 seconds. No more memorizing obscure flags for tar, find, or awk.",
    example: {
      input: "compress all files modified today into backup.tar.gz",
      output: "find . -type f -mtime 0 -print0 | tar -czvf backup.tar.gz --null -T -"
    }
  }
];
