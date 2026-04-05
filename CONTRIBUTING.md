# Contributing to Gemini Audio MCP

Thank you for your interest in contributing! We welcome contributions from the community to help make this MCP server even better.

## How to Contribute

### 1. Report Bugs
If you find a bug, please open an issue on GitHub. Include:
- A clear description of the problem.
- Steps to reproduce the issue.
- Your environment (OS, Rust version, FFmpeg version).

### 2. Suggest Features
Have an idea for a new tool or parameter? Open an issue to discuss it.

### 3. Submit Pull Requests
1. **Fork the repository**.
2. **Create a branch**: `git checkout -b feature/your-feature-name`.
3. **Make your changes**: Ensure your code follows the project's style and passes all tests.
4. **Run tests**: `cargo test` and `cargo clippy`.
5. **Commit your changes**: Use [Conventional Commits](https://www.conventionalcommits.org/).
6. **Push to your fork**: `git push origin feature/your-feature-name`.
7. **Open a Pull Request**: Provide a clear description of your changes.

## Development Setup

- **Rust**: Latest stable version.
- **FFmpeg**: Required for audio processing.
- **Environment**: Set `GEMINI_API_KEY` for testing API-related features.

## Code Style

- Use `cargo fmt` to format your code.
- Avoid `.unwrap()` in production code; use `anyhow` or `thiserror` for structured error handling.
- Add documentation comments to public functions.

## Licensing

By contributing to this project, you agree that your contributions will be licensed under the MIT License.
