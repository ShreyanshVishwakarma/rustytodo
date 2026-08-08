# Taste
- Prefers delegating work to subagents in parallel to maximize throughput ("delegate tasks to subagents", "do a lot of things at once", "start implementation using subagents", "setup subagents to work on these features parallely"). Confidence: 0.95
- Prefers immediate action over deliberation — wants the agent to "fire up and start implementation" rather than over-analyze or plan excessively ("you are thinking too much"). Confidence: 0.8
- Prefers ambitious, rapid, large-scale improvements over incremental changes when leveling up a codebase ("level up massively as quickly as possible"). Confidence: 0.7
- Prefers task delegation to be done through the Command Code CLI (`cmd`) launched in the terminal, explicitly directing the agent to "write cmd into the terminal... use it to delegate tasks" rather than relying on in-session subagent mechanisms. Confidence: 0.95
- Expects the agent to persist through failures — retrying rather than stopping and reporting the error as final ("and ? you decided to give up ?"). Giving up after a failed attempt is treated as unsatisfactory. Confidence: 0.8
- Wants the agent to actively verify that delegated subagent/session work is actually progressing and producing output — checking processes, logs, and worktrees rather than merely reporting that sessions were launched ("see if the subagents are working or not"). Confidence: 0.65
