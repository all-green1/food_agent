import sys
import os
import json
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from openai import OpenAI
from dotenv import load_dotenv
import json
from react import collect_food_info
from prompts import conversational_orchestrator_prompt
from food_agent.food_agent import PyCommandHandler
from tools import tools

load_dotenv()

client = OpenAI(api_key=os.getenv("OPENAI_API_KEY"))
handler = PyCommandHandler()

# Store food collection sessions
food_collection_sessions = {}



def handle_function_call(function_name, arguments, session_id=None):
    """Handle the execution of called functions"""
    print(f"DEBUG: Function called: {function_name}")
    print(f"DEBUG: Arguments: {arguments}")
    print(f"DEBUG: Session ID: {session_id}")
    
    try:
        if function_name == "collect_food_info":
            print("DEBUG: Starting collect_food_info function")
            # Start new food collection session - user_input will be handled through session management
            if session_id and session_id in food_collection_sessions:
                print("DEBUG: Found existing food collection session")
                # This shouldn't happen since collect_food_info is only called to START a session
                session_state = food_collection_sessions[session_id]
                session_state, response, is_complete = collect_food_info(None, session_state)
            else:
                print("DEBUG: Creating new food collection session")
                # Start new food collection session
                session_state, initial_question, is_complete = collect_food_info()
                if session_id:
                    food_collection_sessions[session_id] = session_state
                    print(f"DEBUG: Stored session state for session {session_id}")
                print(f"DEBUG: Initial question: {initial_question}")
                return f"🍎 Food Collection Started: {initial_question}"
            
        elif function_name == "get_all_food":
            print("DEBUG: Executing get_all_food function")
            result = handler.view_all_food()
            print(f"DEBUG: get_all_food result: {result}")
            return f" Food inventory: {result}"
            
        elif function_name == "advanced_search":
            keyword = arguments.get("keyword", "")
            field = arguments.get("field", "name")
            print(f"DEBUG: Executing advanced_search with keyword='{keyword}', field='{field}'")
            result = handler.search_storage(keyword, field)
            print(f"DEBUG: advanced_search result: {result}")
            return f" Search results: {result}"
            
        else:
            print(f"DEBUG: Unknown function called: {function_name}")
            return f" Unknown function: {function_name}"
            
    except Exception as e:
        print(f"DEBUG: Exception in handle_function_call: {str(e)}")
        return f" Error executing {function_name}: {str(e)}"

def process_message(user_input, conversation_history=None, session_id=None, user_context=None):
    """Process user message and return AI response"""
    print(f"DEBUG: process_message called with session_id: {session_id}")
    print(f"DEBUG: User context: {user_context}")
    
    if conversation_history is None:
        conversation_history = [{"role": "system", "content": conversational_orchestrator_prompt}]
    
    # Check if this is a food collection session continuation
    if session_id and session_id in food_collection_sessions:
        print(f"DEBUG: Found active food collection session for {session_id}")
        
        try:
            # This is a food collection response - process directly
            session_state = food_collection_sessions[session_id]
            
            # Prepare user context with Google token if available
            enhanced_user_context = user_context.copy() if user_context else {}
            if user_context and user_context.get("user_id"):
                # Get Google token from app.py context (we'll need to pass this)
                enhanced_user_context["google_token"] = user_context.get("google_token")
            
            session_state, response, is_complete = collect_food_info(user_input, session_state, enhanced_user_context)
            
            print(f"DEBUG: collect_food_info returned - response: '{response}', is_complete: {is_complete}")
            
            # Ensure response is never None/empty
            if not response or (isinstance(response, str) and response.strip() == ""):
                print("DEBUG: Empty response detected, using fallback")
                response = "I'm processing your food information. Please continue."
            
            if is_complete:
                print(f"DEBUG: Food collection completed for session {session_id}")
                # Clean up completed session
                del food_collection_sessions[session_id]
                
                # The calendar integration is now handled in the Rust backend
                # It will either create a calendar event or return calendar links
                assistant_response = response
            else:
                print(f"DEBUG: Food collection continuing for session {session_id}")
                # Update session and continue collection
                food_collection_sessions[session_id] = session_state
                assistant_response = f"🍎 {response}"
                
            print(f"DEBUG: Final assistant response: {assistant_response}")
            conversation_history.append({"role": "user", "content": user_input})
            conversation_history.append({"role": "assistant", "content": assistant_response})
            return assistant_response, conversation_history
            
        except Exception as e:
            print(f"DEBUG: Exception in food collection flow: {str(e)}")
            print(f"DEBUG: Exception type: {type(e)}")
            import traceback
            print(f"DEBUG: Full traceback: {traceback.format_exc()}")
            
            # Clean up session and provide fallback response
            if session_id in food_collection_sessions:
                del food_collection_sessions[session_id]
            
            assistant_response = "I encountered an issue processing your food information. Let's start over. What food would you like to add?"
            conversation_history.append({"role": "user", "content": user_input})
            conversation_history.append({"role": "assistant", "content": assistant_response})
            return assistant_response, conversation_history
    
    # Add user input to conversation history
    conversation_history.append({"role": "user", "content": user_input})
    
    # STEP 1: Check if this is food-related WITHOUT exposing tools
    # This prevents tool availability bias
    print("DEBUG: Step 1 - Checking if request is food-related (no tools exposed)")
    
    # Create a classification prompt that forces explicit food vs non-food decision
    classification_prompt = [
        {"role": "system", "content": """You are a food topic classifier. Your ONLY job is to determine if a user request is food-related or not.

Respond with EXACTLY ONE WORD:
- "FOOD" if the request is about food inventory, nutrition, cooking, meal planning, or any food-related topic
- "NON-FOOD" if the request is about anything else (coding, weather, general help, etc.)

Do not provide any other response. Just the single word classification."""},
        {"role": "user", "content": user_input}
    ]
    
    classification_response = client.chat.completions.create(
        model="gpt-4o-mini",
        messages=classification_prompt,
        tools=None,  # No tools - prevents tool availability bias
        temperature=0.1  # Low temperature for consistent classification
    )
    
    classification = classification_response.choices[0].message.content.strip().upper()
    print(f"DEBUG: Classification result: '{classification}'")
    
    if classification == "NON-FOOD":
        # This is a non-food request - generate refusal using original prompt
        print(f"DEBUG: Conversation history length {len(conversation_history)}")
        print("DEBUG: Non-food request detected, generating refusal")
        print(f"DEBUG: Conversation history being sent to refusal generation:")
        for i, msg in enumerate(conversation_history):
            print(f"  [{i}] Role: {msg['role']}, Content: {msg['content'][:100]}...")
        refusal_response = client.chat.completions.create(
            model="gpt-4o-2024-08-06",
            messages=conversation_history,
            temperature=0.1
        )
        print(f"DEBUG: Conversation history length is {len(conversation_history)}")
        refusal_message = refusal_response.choices[0].message.content
        print(f"DEBUG: Refusal message: '{refusal_message}'")
        conversation_history.append({"role": "assistant", "content": refusal_message})
        return refusal_message, conversation_history
    
    # STEP 2: This appears to be food-related, now expose tools for function calling
    print("DEBUG: Step 2 - Food-related request detected, exposing tools")
    response = client.chat.completions.create(
        model="gpt-4o-mini",
        messages=conversation_history,
        tools=tools,  # Now provide tools since it's food-related
        tool_choice="auto",
        temperature=0.7
    )
    
    message = response.choices[0].message
    
    # Check if the model wants to call a function
    if message.tool_calls:
        print(f"DEBUG: Model wants to call {len(message.tool_calls)} tool(s)")
        # Add assistant message to conversation
        conversation_history.append(message)
        
        for tool_call in message.tool_calls:
            function_name = tool_call.function.name
            arguments = json.loads(tool_call.function.arguments)
            
            print(f"DEBUG: Executing tool call - function: {function_name}, arguments: {arguments}")
            
            # Execute the function
            function_result = handle_function_call(function_name, arguments, session_id)
            
            print(f"DEBUG: Tool call result: {function_result}")
            
            # Add function result to conversation
            conversation_history.append({
                "tool_call_id": tool_call.id,
                "role": "tool",
                "name": function_name,
                "content": function_result
            })
        
        # Get the final response after function execution
        print("DEBUG: Getting final response after function execution")
        final_response = client.chat.completions.create(
            model="gpt-4o-mini",
            messages=conversation_history,
            tools=tools,
            tool_choice="auto",
            temperature=0.7
        )
        
        assistant_response = final_response.choices[0].message.content
        print(f"DEBUG: Final response from LLM: '{assistant_response}'")
        
        # Ensure response is never None/empty
        if not assistant_response or assistant_response.strip() == "":
            print("DEBUG: Empty final response detected, using fallback")
            assistant_response = "I understand you want to work with food items. How can I help you today?"
            
        conversation_history.append({"role": "assistant", "content": assistant_response})
        
    else:
        print("DEBUG: No tool calls, generating regular chat response")
        # Regular chat response
        assistant_response = message.content
        print(f"DEBUG: Regular chat response: '{assistant_response}'")
        
        # Ensure response is never None/empty
        if not assistant_response or assistant_response.strip() == "":
            print("DEBUG: Empty regular response detected, using fallback")
            assistant_response = "I'm here to help with your food management. What would you like to do?"
            
        conversation_history.append({"role": "assistant", "content": assistant_response})
    
    # Keep conversation history manageable (last 15 exchanges)
    if len(conversation_history) > 31:  # system + 15 exchanges
        conversation_history = [conversation_history[0]] + conversation_history[-30:]
    
    return assistant_response, conversation_history

def create_new_conversation():
    """Create a new conversation with initial system prompt"""
    return [{"role": "system", "content": conversational_orchestrator_prompt}]

def get_active_food_collection_sessions():
    """Get list of active food collection session IDs"""
    return list(food_collection_sessions.keys())

def cleanup_food_collection_session(session_id):
    """Manually cleanup a food collection session"""
    if session_id in food_collection_sessions:
        del food_collection_sessions[session_id]
        return True
    return False 