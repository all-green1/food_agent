import sys
import os
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

def collect_food_info(user_input=None, session_state=None):
    """
    Your original collect_food_info function, modified to work with web apps.
    Preserves all your original logic, prompts, and flow.
    """
    # Initialize session state if not provided
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
            "log": [],
            "current_question": "",
            "waiting_for_input": False
        }
    
    food_info = session_state["food_info"]
    count = session_state["count"]
    log = session_state["log"]
    
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
            """.strip()
        
        # Get the question to ask the user
        log.append({"role": "system", "content": llm_prompt})
        step_response = client.responses.create(
            model="gpt-4o-mini-2024-07-18",
            input=log,
            store=False
        )
        question = step_response.output_text.strip()
        log.append({"role": "assistant", "content": question})
        
        session_state["current_question"] = question
        session_state["waiting_for_input"] = True
        session_state["log"] = log
        
        print(f"DEBUG [react.py]: Current food info: {food_info}")
        
        return session_state, question, False
    
    # Process user input (your original logic)
    current_info = "\n".join([f" - {key.replace('_', ' ').title()}: {value if value else 'not yet provided'}"
                              for key, value in food_info.items()])
    
    log.append({"role": "user", "content": user_input})

    # Ask LLM to extract the value from the user's response (your original approach)
    extraction_prompt = f"""
    We asked the user: {session_state.get('current_question', '')}
    They answered: '{user_input}'
    
    Given the information we have already collected:
    {current_info}

    Determine which missing field this response answers and extract the value.
    Reply in the format: field_name=value
    For example: quantity=5kg
    """.strip()

    extraction_response = client.responses.create(
        model="gpt-4o-mini-2024-07-18",
        input=[{"role": "user", "content": extraction_prompt}],
        store=False
    )

    field_and_value = extraction_response.output_text.strip()
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
        confirm_response = client.responses.create(
            model="gpt-4o-mini-2024-07-18",
            input=[{"role": "user", "content": confirmation_prompt}],
            store=False
        )
        confirm_result = confirm_response.output_text.strip().lower()
    
        if "yes" in confirm_response.output_text.strip().lower():
            # Your original database save logic
            result = handler.handle_command(
                command="add",
                name=food_info["name"],
                stock_date=food_info["stock_date"],
                food_type=food_info["food_type"],
                storage_type=food_info["storage_type"],
                quantity=food_info["quantity"],
                expiry_date=food_info["expiry_date"]
            )
            session_state["completed"] = True
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
        """.strip()
    
    # Get the question to ask the user
    log.append({"role": "system", "content": llm_prompt})
    step_response = client.responses.create(
        model="gpt-4o-mini-2024-07-18",
        input=log,
        store=False
    )
    question = step_response.output_text.strip()
    log.append({"role": "assistant", "content": question})
    
    # Update session state
    session_state["food_info"] = food_info
    session_state["log"] = log
    session_state["current_question"] = question
    session_state["waiting_for_input"] = True
    
    print(f"DEBUG [react.py]: Current food info: {food_info}")
    
    return session_state, question, False