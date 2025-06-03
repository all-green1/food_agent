from openai import OpenAI
from dotenv import load_dotenv
import os
import json
from react import collect_food_info
from prompts import conversational_orchestrator_prompt
from food_agent.food_agent import PyCommandHandler
from tools import tools

load_dotenv()

client = OpenAI(api_key=os.getenv("OPENAI_API_KEY"))
handler = PyCommandHandler()

def handle_function_call(function_name, arguments):
    """Handle the execution of called functions"""
    try:
        if function_name == "collect_food_info":
            print("FoodAgent: Great! Let me help you add those items to your inventory.\n")
            collect_food_info()
            return "Food collection process completed successfully."
            
        elif function_name == "get_all_food":
            print("FoodAgent: Let me show you all your food items...\n")
            result = handler.view_all_food()
            return f"Food inventory displayed: {result}"
            
        elif function_name == "advanced_search":
            keyword = arguments.get("keyword", "")
            field = arguments.get("field", "name")
            print(f"FoodAgent: Searching for {keyword} in {field}...\n")
            result = handler.search_storage(keyword, field)
            return f"Search completed: {result}"
            
        else:
            return f"Unknown function: {function_name}"
            
    except Exception as e:
        return f"Error executing {function_name}: {str(e)}"

def main():
    print("🍎 Welcome to FoodAgent! I'm here to help with your food management.")
    print("You can chat with me about anything food-related, or tell me about groceries you want to add.\n")
    
    conversation_history = [
        {"role": "system", "content": conversational_orchestrator_prompt}
    ]
    
    while True:
        try:
            user_input = input("You: ").strip()
            
            if user_input.lower() in ['exit', 'quit', 'bye', 'goodbye']:
                print("FoodAgent: Goodbye! Happy cooking! 🍳")
                break
                
            if not user_input:
                continue
                
            # Add user input to conversation history
            conversation_history.append({"role": "user", "content": user_input})
            
            # Get response from LLM with function calling
            response = client.chat.completions.create(
                model="gpt-4o-mini",
                messages=conversation_history,
                tools=tools,
                tool_choice="auto",
                temperature=0.7
            )
            
            message = response.choices[0].message
            
            # Check if the model wants to call a function
            if message.tool_calls:
                # Add assistant message to conversation
                conversation_history.append(message)
                
                for tool_call in message.tool_calls:
                    function_name = tool_call.function.name
                    arguments = json.loads(tool_call.function.arguments)
                    
                    # Execute the function
                    function_result = handle_function_call(function_name, arguments)
                    
                    # Add function result to conversation
                    conversation_history.append({
                        "tool_call_id": tool_call.id,
                        "role": "tool",
                        "name": function_name,
                        "content": function_result
                    })
                
                # Get the final response after function execution
                final_response = client.chat.completions.create(
                    model="gpt-4o-mini",
                    messages=conversation_history,
                    tools=tools,
                    tool_choice="auto",
                    temperature=0.7
                )
                
                assistant_response = final_response.choices[0].message.content
                print(f"FoodAgent: {assistant_response}")
                conversation_history.append({"role": "assistant", "content": assistant_response})
                
            else:
                # Regular chat response
                assistant_response = message.content
                print(f"FoodAgent: {assistant_response}")
                conversation_history.append({"role": "assistant", "content": assistant_response})
            
            # Keep conversation history manageable (last 15 exchanges)
            if len(conversation_history) > 31:  # system + 15 exchanges
                conversation_history = [conversation_history[0]] + conversation_history[-30:]
                
        except KeyboardInterrupt:
            print("\nFoodAgent: Goodbye! Happy cooking! 🍳")
            break
        except Exception as e:
            print(f"FoodAgent: Sorry, I encountered an error: {e}")
            print("Let's try again!")

if __name__ == "__main__":
    main()

