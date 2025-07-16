# 🍽️ AI-Powered Food Management Agent

A conversational AI assistant that helps individual users manage their food inventory, reduce waste, and make smarter food decisions — all through natural language interaction.

---

## 💡 Project Overview

This project is a generative AI-powered food assistant designed to help **individual consumers** track what food they have, get reminders before items expire, discover recipe ideas, and reduce food waste — all through a chat-based interface.

Users can:
- Add new food items to their inventory
- Check what's available
- Delete or update food items
- Get personalized meal suggestions
- Search by category, expiry, or keyword
- Talk to a fallback NLP-based system if the agent gets confused

The system uses a structured food database under the hood, and includes intelligent fallback mechanisms for robust user experience.

---

## How this Relates to Industry Use Cases
Unlike enterprise waste tracking solutions that focus on disposal analytics (e.g., using computer vision in commercial kitchens), this project tackles food waste before it happensby helping individuals stay aware of what they already have and how to use it.
It complements industry efforts by building behavior change tools at the consumer level.

## ✨ Key Features

### 🤖 Conversational Agent
- Built on top of a generative language model
- Handles multiple user intents through natural language
- Agentic memory tied to structured food inventory

### 🗂️ Structured Inventory System
- Stores food items with fields like name, quantity, unit, expiry date
- Supports search, CRUD operations, and tracking expiry

### 🛡️ Robust Fallback Pipeline
- NLP-based fallback collector for situations where the LLM fails or times out
- Keeps the experience consistent and resilient

### 🔍 Food Search and Discovery
- Users can ask:  
  - "What do I have in the fridge?"  
  - "List all dairy products"  
  - "Show me food expiring this week"  
  - "Suggest a meal with tomatoes and rice"

### ⚙️ Developer-Friendly Design
- Modular and extensible backend
- Separation of agent logic and inventory logic
- Designed to be API-deployable and memory-persistent

---

## 🏗️ Tech Stack

- **LLM backend**: OpenAI / Local model (customizable)
- **Fallback NLP**: Regex-based
- **Database**: MySQL
- **Agent logic**: Python
- **Inventory engine**: Rust (via PyO3 FFI bridge)
- **Deployment**: Docker-ready, extensible to cloud environments (AWS, etc.)

---

## Possible Extensions
- Grocery list integration
- Personal Food insights dashboard
- Meal Planning

## Contact
Feel free to reach out if you're interested in:
- Collaborating on foodtech/AI solutions
- Using this project in a research or commercial context
- Just talking shop about agents, LLMs, and structured automation

Jeremiah Akisanya
LinkedIn: www.linkedin.com/in/jeremiah-akisanya
Email: akisanyajeremiah@gmail.com

## 🚀 Getting Started

### Prerequisites

- Python 3.9+
- [Rust (optional, for inventory engine)](https://www.rust-lang.org/tools/install)
- OpenAI API key or local model weights
- [Poetry](https://python-poetry.org/) or `pipenv` (recommended)

### Clone the repo

```bash
git clone https://github.com/all-green1/food_agent.git
cd your_project_name

### Install Dependencies

poetry install
# or
pip install -r requirements.txt
