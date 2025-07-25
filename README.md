# 🧠 AI Memory Engine

**Persistent project memory for AI coding assistants - No more explaining the same codebase repeatedly.**

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-8%2F8%20passing-green.svg)](#testing)

## 🎯 Problem Solved

Current AI coding assistants suffer from "AI amnesia" - they forget your codebase context between conversations, forcing you to repeatedly explain the same code structure, dependencies, and patterns. **AI Memory Engine** solves this by providing persistent, local project memory.

## ✨ Features

- 🧠 **Persistent Memory** - Remembers your project structure across sessions
- 🔍 **Smart Code Analysis** - Extracts functions, classes, imports, and variables
- 🚀 **Multi-Language Support** - JavaScript, TypeScript, Python (more coming)
- 💻 **Local-First** - No cloud dependencies, works offline
- ⚡ **Fast & Lightweight** - Rust core for performance
- 🛠️ **Developer-Friendly** - Simple CLI interface

## 🚀 Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/jahboukie/aimemoryengine.git
cd aimemoryengine

# Build the project
cargo build --release

# The binary will be available at target/release/aimemoryengine
```

### Usage

```bash
# Initialize memory tracking for your project
aimemoryengine init

# Analyze a specific file
aimemoryengine analyze src/main.js

# Check memory status
aimemoryengine status

# Query project context
aimemoryengine query "function"

# Reset project memory
aimemoryengine reset
```

## 📊 Example Output

```bash
$ aimemoryengine analyze src/components/App.js

🔬 Analyzing file: src/components/App.js

📊 Analysis Results:
Entities found: 4
Relationships found: 0

🔍 Entities:
  import react at line 1
  import useState at line 2
  class App at line 4
  function handleClick at line 12
```

## 🏗️ Architecture

- **Core Engine** (Rust) - High-performance memory management and code parsing
- **CLI Interface** (Node.js) - User-friendly command-line interface
- **Local Storage** - SQLite database for persistent memory (coming in Week 3)
- **Parser Engine** - Regex-based AST parsing (tree-sitter upgrade planned)

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run specific test suite
cargo test -p memory-engine
```

**Current Status: 8/8 tests passing ✅**

## 🛣️ Roadmap

### ✅ Phase 1-2: Foundation (Complete)
- [x] Core memory data structures
- [x] Multi-language code parsing
- [x] CLI interface
- [x] Professional dependency management

### 🚧 Phase 3: Persistence (In Progress)
- [ ] SQLite integration
- [ ] Query optimization
- [ ] Memory persistence across sessions

### 📋 Phase 4: AI Integration
- [ ] REST API for AI assistants
- [ ] Context window optimization
- [ ] Learning from AI interactions

## 🤝 Contributing

We welcome contributions! This project is being developed using dogfooding - we use the memory engine to build itself.

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `cargo test`
5. Submit a pull request

## 📄 License

MIT License - see [LICENSE](LICENSE) file for details.

## 🎯 Target Audience

Built for **serious developers** who want AI assistants that actually understand their codebase. Perfect for:

- 🏢 Professional development teams
- 🚀 Startup engineers
- 🔬 Open source maintainers
- 💻 Individual developers working on complex projects

## 🔧 Development Status

**Current Version**: MVP (Minimum Viable Product)  
**Development Stage**: Week 2 Complete, Week 3 In Progress  
**Stability**: Alpha - suitable for testing and feedback

---

**Built with ❤️ for developers who deserve better AI coding tools.**
