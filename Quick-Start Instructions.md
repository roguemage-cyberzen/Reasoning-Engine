## 💻 Quick Start
1. System Preparation: Ensure Python 3.10+ and pip are installed:

``` Bash
sudo apt update
sudo apt install python3 python3-pip python3-venv -y
```
2. Repository Setup: Create a project directory and initialize a virtual environment to prevent dependency conflicts:

``` Bash
mkdir reasoning_engine_project && cd reasoning_engine_project
python3 -m venv venv
source venv/bin/activate
```

Library Deployment: Save the library code as ReasoningKit.py and your implementation scripts (e.g., php_experiment.py) in the same directory. No external heavy-weight dependencies (like PySAT) are required, as this is a "from-scratch" implementation.
