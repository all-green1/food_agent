from pydantic import BaseModel, Field
from openai import OpenAI
import os
from dotenv import load_dotenv
from prompts import system_prompt, food_type_prompt, quantity_prompt, stock_date_prompt, expiry_date_prompt, storage_type_prompt, food_prompt, assert_food_prompt
import re
from food_agent.food_agent import PyCommandHandler

load_dotenv()

client = OpenAI(api_key=os.getenv("OPENAI_API_KEY"))

class RequestType(BaseModel):
    type: str = Field(
        description = "A single word classification of what the user would like to do"
    )

class TypeRequest(BaseModel):
    type: str = Field(
        description = "The type of food the user wants to add"
    )

def persistent_querying(list_of_properties: list, base_prompt: str, system_prompt: str, user_input=None, session_data=None):
    """
    Your original persistent_querying function, modified for web apps.
    Preserves all your original logic and approach.
    """
    if session_data is None:
        session_data = {
            "prop": "",
            "response_count": 0,
            "history": [{"role": "system", "content": system_prompt}],
            "current_prompt": base_prompt
        }
    
    if user_input is None:
        return session_data["current_prompt"], session_data, False
    
    session_data["history"].append({"role": "user", "content": user_input})
    response = client.responses.create(
        model="gpt-4o-mini-2024-07-18",
        input=session_data["history"],
        store=False
    )
    prop = response.output_text.strip().lower()
    session_data["history"].append({"role": "assistant", "content": response.output_text.strip()})
    session_data["response_count"] += 1
    session_data["prop"] = prop
    session_data["current_prompt"] = prop
    
    if prop in list_of_properties:
        return prop, session_data, True
    else:
        return prop, session_data, False

class add_food():
    def __init__(self):
        self.typePrompt = food_type_prompt
        self.handler = PyCommandHandler()

    def get_funnel(self, user_input=None, session_data=None):
        """Your original get_funnel logic, modified for web apps"""
        funnels = ["add", "search", "view all", "exit"]
        prompt = system_prompt
        init_prompt = "What would you like to do today?"
        return persistent_querying(funnels, init_prompt, prompt, user_input, session_data)
        
    def get_food_type(self, user_input=None, session_data=None):
        """Your original get_food_type logic, modified for web apps"""
        food_types = ["vegetable", "fruit", "beverage", "grains", "breakfast-cereal", "meat", "dairy", "non-dairy", "edible-oils"]
        prompt = food_type_prompt
        init_prompt = "what type of food is it?"
        return persistent_querying(food_types, init_prompt, prompt, user_input, session_data)

    def get_storage_type(self, user_input=None, session_data=None):
        """Your original get_storage_type logic, modified for web apps"""
        storage_types = ["cold", "warm"]
        prompt = storage_type_prompt
        init_prompt = "How is the food being stored, are you using a cold or warm storage device?"
        return persistent_querying(storage_types, init_prompt, prompt, user_input, session_data)

    def add_new_food(self, step_data=None):
        """
        Your original add_new_food logic, modified for web apps.
        Preserves your exact step-by-step flow.
        """
        if step_data is None:
            # Initialize with your original flow
            step_data = {
                "step": "name",
                "prompt": "What food do you want me to add?",
                "session_data": {},
                "collected": {},
                "complete": False
            }
        
        current_step = step_data["step"]
        user_input = step_data.get("user_input")
        collected = step_data["collected"]
        session_data = step_data.get("session_data", {})
        
        try:
            if current_step == "name":
                result, new_session, is_complete = get_food_name(food_prompt, assert_food_prompt, user_input, session_data.get("name"))
                if is_complete:
                    collected["name"] = result
                    step_data["step"] = "food_type"
                    step_data["prompt"] = "what type of food is it?"
                    session_data["name"] = new_session
                else:
                    step_data["prompt"] = result
                    session_data["name"] = new_session
                    
            elif current_step == "food_type":
                result, new_session, is_complete = self.get_food_type(user_input, session_data.get("food_type"))
                if is_complete:
                    collected["food_type"] = result
                    step_data["step"] = "quantity"
                    step_data["prompt"] = "How much did you get?"
                    session_data["food_type"] = new_session
                else:
                    step_data["prompt"] = result
                    session_data["food_type"] = new_session
                    
            elif current_step == "quantity":
                result, new_session, is_complete = get_food_quantity(quantity_prompt, user_input, session_data.get("quantity"))
                if is_complete:
                    collected["quantity"] = result
                    step_data["step"] = "stock_date"
                    step_data["prompt"] = "When did you get this food? (e.g. today, yesterday, 24-05-2025): "
                    session_data["quantity"] = new_session
                else:
                    step_data["prompt"] = result
                    session_data["quantity"] = new_session
                    
            elif current_step == "stock_date":
                result, new_session, is_complete = get_stock_date(stock_date_prompt, user_input, session_data.get("stock_date"))
                if is_complete:
                    collected["stock_date"] = result
                    step_data["step"] = "expiry_date"
                    step_data["prompt"] = "Is there an expiry date for this food? If yes, enter the date (e.g. 24-05-2025). Otherwise, enter 'none': "
                    session_data["stock_date"] = new_session
                else:
                    step_data["prompt"] = result
                    session_data["stock_date"] = new_session
                    
            elif current_step == "expiry_date":
                result, new_session, is_complete = get_expiry_date(expiry_date_prompt, user_input, session_data.get("expiry_date"))
                if is_complete:
                    collected["expiry_date"] = result
                    step_data["step"] = "storage_type"
                    step_data["prompt"] = "How is the food being stored, are you using a cold or warm storage device?"
                    session_data["expiry_date"] = new_session
                else:
                    step_data["prompt"] = result
                    session_data["expiry_date"] = new_session
                    
            elif current_step == "storage_type":
                result, new_session, is_complete = self.get_storage_type(user_input, session_data.get("storage_type"))
                if is_complete:
                    collected["storage_type"] = result
                    # Your original database save logic
                    db_result = self.handler.add_food(
                        name=collected["name"],
                        stock_date=collected["stock_date"],
                        food_type=collected["food_type"],
                        storage_type=collected["storage_type"],
                        quantity=collected["quantity"],
                        expiry_date=collected["expiry_date"]
                    )
                    step_data["complete"] = True
                    step_data["prompt"] = str(db_result)
                    session_data["storage_type"] = new_session
                else:
                    step_data["prompt"] = result
                    session_data["storage_type"] = new_session
                    
            step_data["session_data"] = session_data
            step_data["collected"] = collected
            return step_data
            
        except Exception as e:
            step_data["prompt"] = f"Error adding food: {e}"
            return step_data

    def view_all_food(self):
        """Your original view_all_food logic - returns result instead of printing"""
        try:
            result = self.handler.view_all_food()
            return result
        except Exception as e:
            return f"Error viewing food: {e}"

    def search_food(self, query: str, field: str):
        """Your original search_food logic - returns result instead of printing"""
        try:
            result = self.handler.advanced_search(query, field)
            return result
        except Exception as e:
            return f"Error searching food: {e}"

def get_food_name(system_prompt: str, assertion_prompt: str, user_input=None, session_data=None):
    """
    Your original get_food_name logic, modified for web apps.
    Preserves your validation approach and while loop logic.
    """
    if session_data is None:
        session_data = {
            "history": [{"role": "system", "content": system_prompt}],
            "name": None,
            "attempts": 0
        }
    
    if user_input is None:
        init_prompt = "What food do you want me to add?"
        session_data["history"].append({"role": "system", "content": init_prompt})
        return init_prompt, session_data, False
    
    session_data["history"].append({"role": "user", "content": user_input})
    call_1 = client.responses.create(
        model="gpt-4o-mini-2024-07-18",
        input=session_data["history"],
        store=False
    )
    prop = call_1.output_text.strip().lower()

    validation_prompt = [{"role": "system", "content": assertion_prompt},
                         {"role": "user", "content": prop}]
    call_2 = client.responses.create(
        model="gpt-4o-mini-2024-07-18",
        input=validation_prompt,
        store=False
    )
    is_valid = call_2.output_text.strip().lower()
    
    if is_valid == "yes":
        session_data["name"] = prop
        return prop, session_data, True
    else:
        session_data["attempts"] += 1
        return "That doesn't seem like a valid food. Let's try again.\nWhat food do you want me to add?", session_data, False

def get_food_quantity(system_prompt: str, user_input=None, session_data=None):
    """Your original get_food_quantity logic, modified for web apps"""
    pattern = r"\d+(l|g)"
    
    if session_data is None:
        session_data = {
            "history": [{"role": "system", "content": system_prompt}],
            "prop": ""
        }
    
    if user_input is None:
        return "How much did you get?", session_data, False
    
    session_data["history"].append({"role": "user", "content": user_input})
    response = client.responses.create(
        model="gpt-4o-mini-2024-07-18",
        input=session_data["history"],
        store=False
    )
    prop = response.output_text.strip().lower()
    session_data["history"].append({"role": "assistant", "content": response.output_text.strip()})
    session_data["prop"] = prop
    
    if re.match(pattern, prop):
        output = response.output_text.strip().lower()
        match = re.match(pattern, output)
        if match:
            return match.group(), session_data, True
    
    return prop, session_data, False
    
def get_stock_date(system_prompt: str, user_input=None, session_data=None):
    """Your original get_stock_date logic, modified for web apps"""
    pattern = r"^(today|yesterday|\d{2}-\d{2}-\d{4})$"
    
    if session_data is None:
        session_data = {
            "history": [{"role": "system", "content": system_prompt}],
            "prop": ""
        }
    
    if user_input is None:
        return "When did you get this food? (e.g. today, yesterday, 24-05-2025): ", session_data, False
    
    session_data["history"].append({"role": "user", "content": user_input})
    response = client.responses.create(
        model="gpt-4o-mini-2024-07-18",
        input=session_data["history"],
        store=False
    )
    prop = response.output_text.strip().lower()
    session_data["history"].append({"role": "assistant", "content": response.output_text.strip()})
    session_data["prop"] = prop
    
    # Your original regex check
    if re.match(pattern, prop):
        output = response.output_text.strip().lower()
        match = re.match(pattern, output)
        if match:
            return match.group(), session_data, True
    
    return prop, session_data, False
    
def get_expiry_date(system_prompt: str, user_input=None, session_data=None):
    """Your original get_expiry_date logic, modified for web apps"""
    pattern = r"^(none|\d{2}-\d{2}-\d{4})$"
    
    if session_data is None:
        session_data = {
            "history": [{"role": "system", "content": system_prompt}],
            "prop": ""
        }
    
    if user_input is None:
        return "Is there an expiry date for this food? If yes, enter the date (e.g. 24-05-2025). Otherwise, enter 'none': ", session_data, False
    
    session_data["history"].append({"role": "user", "content": user_input})
    response = client.responses.create(
        model="gpt-4o-mini-2024-07-18",
        input=session_data["history"],
        store=False
    )
    prop = response.output_text.strip().lower()
    session_data["history"].append({"role": "assistant", "content": response.output_text.strip()})
    session_data["prop"] = prop
    
    if re.match(pattern, prop):
        output = response.output_text.strip().lower()
        match = re.match(pattern, output)
        if match:
            return match.group(), session_data, True
    
    return prop, session_data, False


add_new_food = add_food()

if __name__ == "__main__":
    
    print("Note: This module is now web-compatible. Use the functions with user_input parameters.")
    print("Original terminal mode preserved for testing:")
    
    while True:
        funnel_result, funnel_session, funnel_complete = add_new_food.get_funnel()
        print(funnel_result)
        
        if funnel_complete:
            funnel = funnel_result
            if funnel == "add":
                print(add_new_food.add_new_food())
            elif funnel == "view all":
                print(add_new_food.view_all_food())
            elif funnel == "search":
                query = input("Enter search query: ")
                field = input("Enter field to search: ")
                print(add_new_food.search_food(query, field))
            elif funnel == "exit":
                print("Goodbye!")
                break
