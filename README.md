# 🗂️ shodh

> **A blazing-fast, smart, fuzzy file finder for your terminal — built in Rust.**

---

## 🚀 Project Name
**shodh** *(शोध)*  
"Search" in Sanskrit, Marathi, Hindi, and several Indian languages — symbolizing depth, exploration, and clarity.

---

## 🧠 Description
`shodh` is a command-line utility that performs **fuzzy searching across directory trees**. It's like `fzf`, but deeply hackable, built from scratch in Rust, and with learning as the core motivation. Unlike traditional search tools, `shodh` allows partial, out-of-order, and inexact matches — giving you smart, ranked suggestions of file or folder paths even when you don't remember exact names.

---

## 📦 Features
- 🔍 **Fuzzy matching** of file and folder names (Smith-Waterman algorithm)
- 🧠 **Custom scoring system** for ranking results (exact/prefix matches always top)
- ⚡ **Fast directory traversal** (parallelized with Rayon)
- 🛠️ Simple CLI interface, easily embeddable in shell scripts
- 🎨 **Aesthetic, colorized output**
- 🧪 Extensible matcher logic for advanced heuristics
- 🏗️ Modular, robust, single-file design

---

## 🖥️ Installation

```sh
# Clone the repo
$ git clone https://github.com/urngmi/shodh.git
$ cd shodh

# Build with Cargo
$ cargo build --release

# Run (from project root or add to your PATH)
$ ./target/release/shodh <query> [root_dir]
```

---

## 🧩 CLI Usage

```sh
shodh [FLAGS] <query> [root_dir]
```

### Flags
| Flag                  | Description                                 |
|-----------------------|---------------------------------------------|
| `-h`, `--help`        | Show help message                           |
| `-v`, `--version`     | Show version info                           |
| `-n`, `--num <N>`     | Limit number of results (default: 10)       |
| `--files-only`        | Only show files                             |
| `--dirs-only`         | Only show directories                       |
| `-i`, `--ignore-case` | Case-insensitive search (default)           |
| `-s`, `--case-sensitive` | Case-sensitive search                    |
| `--no-parallel`       | Disable parallel scoring                    |

### Examples
```sh
shodh kilo src --files-only -n 20
shodh resume ~/Documents --dirs-only
shodh "UPI" / --case-sensitive --num 5
```

---

## ✨ Example Output
```
Results:
[10012] DIR  /home/omkarb/Desktop/Projects/resume
[  512] FILE /home/omkarb/Documents/old_resume.pdf
[  412] FILE /var/tmp/resume_draft.txt
```

---

## 🤝 Contributing
Pull requests, issues, and suggestions are welcome! For major changes, please open an issue first to discuss what you would like to change.

---

## 📄 License
MIT 