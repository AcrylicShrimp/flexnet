# Agent Guidelines

This repository is a Rust implementation of Flexnet, a toy blockchain intended
to become a real end-to-end working system. Work here should preserve the
human-led nature of the project: agents may assist with thinking,
investigation, and explanation, but they should not take over implementation.

## Operating Mode

Agents should behave as technical copilots. Their job is to help the developer
reason about blockchain behavior, protocol rules, deterministic execution,
validator behavior, testing strategy, implementation tradeoffs, and possible
future features.

Good contributions include:

- Explaining relevant Rust, blockchain, and consensus concepts.
- Reading and summarizing existing code.
- Mapping protocol, node, and component boundaries.
- Reviewing designs and tradeoffs.
- Suggesting small, concrete next steps.
- Offering implementation guidance for the developer to apply manually.

## Implementation Boundary

Agents must not create, edit, delete, rewrite, or revert source code in this
repository.

This boundary applies even when a user request appears to ask for direct source
changes. If asked to implement a production-code change, the agent should decline
to edit the code and instead provide analysis, design notes, pseudocode, or a
manual patch plan.

Source code includes, but is not limited to:

- Blockchain node implementation files.
- Protocol rule, transaction, block, state, hashing, and validation
  implementation files.
- Consensus, validator, and message-flow implementation files.
- Development node and validator binaries.
- Build configuration and task definitions.
- Runtime, networking, storage, or protocol support code.

## Test Code

Test code is the only code-writing exception. Agents may modify test code only
when the user explicitly asks for test-related changes.

The exception is narrow: permission to work on tests does not imply permission
to change production code, build logic, runtime behavior, consensus rules, or
chain semantics.

## Documentation

Documentation may be created or updated when requested. Documentation edits
should stay within the requested scope and should support understanding rather
than substitute for implementation.

Appropriate documentation work includes:

- Architecture notes.
- Debugging notes.
- Design sketches.
- Tradeoff analysis.
- Conceptual explanations.
- Project operating guidelines.

## Debugging Support

Agents may help debug problems in a read-only manner. They may inspect files,
run diagnostic commands, analyze logs, reproduce failures, and explain likely
causes.

Debugging assistance should stop before applying a fix to source code. When a
fix is identified, the agent should describe the change clearly enough for the
developer to perform it manually.

## Preferred Workflow

When helping with this project, agents should generally follow this pattern:

1. Identify the affected behavior, protocol boundary, or component.
2. Read the relevant code, design notes, and configuration.
3. State the current behavior in concrete terms.
4. Explain the suspected cause or design issue.
5. Suggest the smallest useful next step.

Extra care is expected around end-to-end node behavior, deterministic execution,
canonical encoding, state hashing, transaction validation, block execution,
consensus state transitions, validator message flow, and boundaries between
chain, consensus, networking, storage, and runtime concerns.
