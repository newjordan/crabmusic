# CrabCrust Installation Guide

## 🚀 **Badabing Badaboom One-Command Install**

Choose your fighter:

### **Windows** 🪟

#### **Option 1: PowerShell Installer** (Recommended!)
```powershell
iwr -useb https://raw.githubusercontent.com/USER/crabcrust/main/install.ps1 | iex
```
✨ Downloads pre-built binary, adds to PATH, ready to go!

#### **Option 2: Scoop** (If you have Scoop)
```powershell
scoop bucket add extras
scoop install crabcrust
```

#### **Option 3: Chocolatey** (Coming soon!)
```powershell
choco install crabcrust
```

#### **Option 4: Winget** (Coming soon!)
```powershell
winget install crabcrust
```

#### **Option 5: Manual Download**
1. Download: https://github.com/USER/crabcrust/releases/latest
2. Extract `crabcrust-windows-x64.exe`
3. Add to PATH

---

### **macOS** 🍎

#### **Option 1: Shell Installer**
```bash
curl -sSL https://raw.githubusercontent.com/USER/crabcrust/main/install.sh | bash
```

#### **Option 2: Homebrew** (Coming soon!)
```bash
brew tap USER/crabcrust
brew install crabcrust
```

---

### **Linux** 🐧

#### **Option 1: Shell Installer**
```bash
curl -sSL https://raw.githubusercontent.com/USER/crabcrust/main/install.sh | bash
```

#### **Option 2: Cargo** (If you have Rust)
```bash
cargo install crabcrust
```

---

## ⚙️ **Setup Git Wrapper** (The Fun Part!)

After installing, set up the git wrapper:

### **PowerShell**
```powershell
# Open your profile
notepad $PROFILE

# Add this function:
function git { crabcrust git $args }

# Save and reload
. $PROFILE
```

### **Bash/Zsh**
```bash
# Add to ~/.bashrc or ~/.zshrc
echo "alias git='crabcrust git'" >> ~/.bashrc

# Reload
source ~/.bashrc
```

### **Fish**
```fish
# Add to ~/.config/fish/config.fish
alias git='crabcrust git'
```

---

## ✅ **Verify Installation**

```bash
# Check it's installed
crabcrust --version

# Test the animations!
crabcrust demo rocket     # 🚀 Watch the rocket launch!
crabcrust demo spinner    # 🌀 Spinning circle
crabcrust demo save       # 💾 Floppy disk save

# Try with git
git status               # Should show spinner animation
```

---

## 🔧 **Requirements**

### **Windows**
- **Windows Terminal** (recommended) or any modern terminal
- Font with Unicode/Braille support:
  - Cascadia Code (comes with Windows Terminal) ✅
  - JetBrains Mono ✅
  - Fira Code ✅

**Test Braille support:**
```powershell
Write-Host "⠀⣿⠁⠂⣾⠃⣽⠄⣼⠅⣻⠆⣺"
```
If you see patterns → ✅ Good!
If you see boxes → ⚠️ Need better terminal/font

### **macOS/Linux**
- Any modern terminal (most work out of the box)
- Unicode font (usually default)

---

## 🎮 **What You Get**

After setup, every git command becomes an arcade game:

```bash
git commit -m "Add feature"   # 💾 Save disk animation!
git push                      # 🚀 ROCKET LAUNCHES!
git pull                      # 🌀 Spinner while downloading
git status                    # 🌀 Quick spinner
```

---

## 🚨 **Troubleshooting**

### "Command not found"
- Make sure crabcrust is in your PATH
- Restart your terminal
- Run the installer again

### "Weird characters instead of animations"
- You need a terminal with Unicode/Braille support
- On Windows: Use **Windows Terminal** (not old CMD)
- Check your font supports Braille characters

### "No animations showing"
- Check you set up the git wrapper (see above)
- Make sure you're running `crabcrust git` not just `git`
- Try `crabcrust demo rocket` to test

### "PowerShell says 'cannot be loaded'"
- Run: `Set-ExecutionPolicy -Scope CurrentUser RemoteSigned`
- Try installer again

---

## 📦 **Uninstall**

### **Windows**
```powershell
Remove-Item "$env:LOCALAPPDATA\crabcrust" -Recurse
# Remove from $PROFILE
```

### **macOS/Linux**
```bash
rm ~/.local/bin/crabcrust
# Remove alias from shell config
```

---

## 🎯 **Advanced: Build from Source**

If you want the latest code:

```bash
# Clone repo
git clone https://github.com/USER/crabcrust.git
cd crabcrust

# Build and install
cargo install --path .
```

---

## 📝 **Next Steps**

1. ✅ Install crabcrust
2. ✅ Set up git wrapper
3. ✅ Test with `crabcrust demo all`
4. 🎮 Enjoy arcade-style git commands!
5. ⭐ Star the repo if you love it!

---

**Need help?** Open an issue: https://github.com/USER/crabcrust/issues

**Made with 🦀 and ✨**
