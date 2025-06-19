import sys
import os
import json
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from openai import OpenAI
from dotenv import load_dotenv
from utils import add_food
from food_agent.food_agent import PyCommandHandler

load_dotenv()

client = OpenAI(api_key=os.getenv("OPENAI_API_KEY"))
handler = PyCommandHandler()
add_init = add_food()
result = None

def collect_food_info(user_input=None, session_state=None, user_context=None):
    """
    Collect food information from user in an interactive manner
    Returns (session_state, response_text, is_complete)
    """
    print(f"DEBUG: collect_food_info called with user_input: '{user_input}', user_context: {user_context}")
    
    if session_state is None:
        session_state = {
            "food_info": {
        "name": None,
        "food_type": None,
        "storage_type": None,
        "stock_date": None,
        "quantity": None,
        "expiry_date": None
            },
            "count": 0,
            "completed": False
        }
    
    food_info = session_state["food_info"]
    count = session_state.get("count", 0)
    
    # If no user input provided, generate first question
    if user_input is None:
        current_info = "\n".join([f" - {key.replace('_', ' ').title()}: {value if value else 'not yet provided'}"
                                  for key, value in food_info.items()])
        
        llm_prompt = f"""
        You are a helpful assistant guiding a user to provide all necessary information about a food item.
        Here's the information we've collected so far:
        {current_info}
        For context: 
        **name** is the name of the food like banana, apple, beef. Strictly return the name of the food like "apple" or "beef" and no additional character.

        **food type** is the type of food that was mentioned like fruit, vegetable, breakfast_cereal, meat, dairy, non_dairy, water and edible oils. Food type
        strictly has to be one of these.
        
        **storage type** is how the food is being stored. Like "cold" - if it is being stored in the refrigerator or other cold storage equipment and "warm"
        if it is not being stored in a cold storage equipment. Storage type strictly has to be either "cold" or "warm"

        **stock date** is the day the food was bought. It can be one of three types - "today", "yesterday" or an actual date of format "DD-MM-YYYY" as in (23-05-2025).
        Stock date strictly has to be one of this three.

        **quantity** is the amount of food that is being added. and there are two valid units - **grams(g)** and **litres(l)**. a valid quantity submission has to be in these
        units - you extract "50g" as in when the user says "50grams" or "5l" as in when the user says "5litres".

        **expiry date** is the day that the food expires if any. A valid submission is a date in the format "DD-MM-YYYY". If the user says there is no expiry date, the 
        valid submission in this case is "none". Expiry date **strictly** has to be a date in the annotated format or "none"
        Your task:
        1. Identify the next missing field.
        2. Ask a natural question to obtain it from the user.
        3. Only ask about one field at a time.
        4. Be conversational and helpful.

        You can prompt the user now.
            
        """
        
        try:
            response = client.chat.completions.create(
            model="gpt-4o-mini-2024-07-18",
                messages=[{"role": "user", "content": llm_prompt}],
                max_tokens=150
            )
            initial_question = response.choices[0].message.content.strip()
            return session_state, initial_question, False
        except Exception as e:
            print(f"DEBUG: Error generating initial question: {e}")
            return session_state, "What food would you like to add to your inventory?", False
    
    # Process user response using LLM to extract structured data
    current_info = "\n".join([f" - {key.replace('_', ' ').title()}: {value if value else 'not yet provided'}"
                              for key, value in food_info.items()])
    
    llm_prompt = f"""
    You are a helpful assistant that extracts food information from user responses.
    
    Current food information collected:
        {current_info}

    User's response: "{user_input}"
    
    Extract one piece of information from the user's response and return it in the format:
    field_name=value
    
    Rules:
    - Only extract ONE piece of information per response
    - For name: just the food name (e.g. "apple", "beef")
    - For food_type: must be one of: fruit, vegetable, breakfast_cereal, meat, dairy, non_dairy, beverage, grains, edible_oils
    - For storage_type: must be "cold" or "warm"  
    - For stock_date: must be "today", "yesterday", or "DD-MM-YYYY"
    - For quantity: must include unit like "50g" or "2l"
    - For expiry_date: must be "DD-MM-YYYY" or "none"
    
    Only respond with field_name=value, nothing else.
    """
    
    try:
        response = client.chat.completions.create(
            model="gpt-4o-mini-2024-07-18",
            messages=[{"role": "user", "content": llm_prompt}],
            max_tokens=50
        )
        field_and_value = response.choices[0].message.content.strip()
    except Exception as e:
        print(f"DEBUG: Error processing user input: {e}")
        session_state["food_info"] = food_info
        return session_state, "I'm having trouble processing that. Could you please try again?", False

    # Parse the field and value
    if "=" in field_and_value:
        field, value = field_and_value.split("=", 1)
        field = field.strip().lower()
        value = value.strip()
        if field in food_info:
            food_info[field] = value
        else:
            session_state["food_info"] = food_info
            return session_state, f"Unrecognized field: {field}", False
    else:
        session_state["food_info"] = food_info
        return session_state, "Couldn't extract a valid field from your response.", False

    # Check if all fields are complete (your original validation)
    if all(food_info.values()):
        confirmation_prompt = f"""
        All required food information has been collected:
        {current_info}

        All fields are filled with valid data. Reply only with YES to preoceed or NO if you notice any formatting issues."""
        confirm_response = client.chat.completions.create(
            model="gpt-4o-mini-2024-07-18",
            messages=[{"role": "user", "content": confirmation_prompt}],
            max_tokens=50
        )
        confirm_result = confirm_response.choices[0].message.content.strip().lower()
    
        if "yes" in confirm_result:
            # Pass user context to the handler for calendar integration
            user_id = user_context.get("user_id") if user_context else None
            google_token_data = user_context.get("google_token") if user_context else None
            google_token_json = json.dumps(google_token_data) if google_token_data else None
            
            result = handler.add_food(
                name=food_info["name"],
                stock_date=food_info["stock_date"],
                food_type=food_info["food_type"],
                storage_type=food_info["storage_type"],
                quantity=food_info["quantity"],
                expiry_date=food_info["expiry_date"],
                user_id=user_id,
                google_token_json=google_token_json,
            )
            session_state["completed"] = True
            
            # Store food info for calendar message generation
            session_state["final_food_info"] = food_info.copy()
            return session_state, f"All fields successfully collected.\n{result}", True
        else:
            # Continue with your original logic
            pass

    # Your original count logic
    count += 1
    session_state["count"] = count
    
    if count == 7:
        # Your original fallback logic
        try:
            fallback_result = add_init.add_new_food()
            session_state["completed"] = True
            return session_state, f"Having trouble collecting food details, let me guide you step by step\n{fallback_result}", True
        except Exception as e:
            return session_state, f"Error in step-by-step guidance: {e}", False

    # Generate next question (your original prompt logic)
    current_info = "\n".join([f" - {key.replace('_', ' ').title()}: {value if value else 'not yet provided'}"
                              for key, value in food_info.items()])
    
    llm_prompt = f"""
    You are a helpful assistant guiding a user to provide all necessary information about a food item.
    Here's the information we've collected so far:
    {current_info}
    
    Your task:
    1. Identify the next missing field that needs to be filled.
    2. Ask a natural, conversational question to obtain that specific information.
    3. Only ask about ONE field at a time.
    4. Be helpful and encouraging.
    
    Generate the next question now.
    """
    
    try:
        response = client.chat.completions.create(
            model="gpt-4o-mini-2024-07-18",
            messages=[{"role": "user", "content": llm_prompt}],
            max_tokens=100
        )
        next_question = response.choices[0].message.content.strip()
        session_state["food_info"] = food_info
        return session_state, next_question, False
    except Exception as e:
        print(f"DEBUG: Error generating next question: {e}")
        session_state["food_info"] = food_info
        return session_state, "What other details can you provide about this food item?", False