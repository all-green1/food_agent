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

def persistent_querying(list_of_properties:list, base_prompt:str, system_prompt:str) -> str:
    prop = ""
    response_count = 0
    history = [{"role": "system", "content": system_prompt}]
    while prop not in list_of_properties:
        user_input = input(base_prompt)
        history.append({"role": "user", "content": user_input})
        response = client.responses.create(
            model="gpt-4o-mini-2024-07-18",
            input=history,
            store=False
        )
        prop = response.output_text.strip().lower()
        history.append({"role": "assistant", "content": response.output_text.strip()})
        response_count += 1
        
        base_prompt = prop

    return prop

class add_food():
    def __init__(self):
        self.typePrompt = food_type_prompt
        self.handler = PyCommandHandler()

    def get_funnel(self):
        funnels = ["add", "search", "view all", "exit"]
        prompt = system_prompt
        init_prompt = "What would you like to do today?"
        funnel = persistent_querying(funnels, init_prompt, prompt)
        return funnel
        
    def get_food_type(self):
        food_types = ["vegetable", "fruit", "beverage", "grains", "breakfast-cereal", "meat", "dairy", "non-dairy", "edible-oils"]
        prompt = food_type_prompt
        init_prompt = "what type of food is it?"
        food_type = persistent_querying(food_types, init_prompt, prompt)
        return food_type

    def get_storage_type(self):
        storage_types = ["cold", "warm"]
        prompt = storage_type_prompt
        init_prompt = "How is the food being stored, are you using a cold or warm storage device?"
        storage_type = persistent_querying(storage_types, init_prompt, prompt)
        return storage_type

    def add_new_food(self):
        name = get_food_name(food_prompt, assert_food_prompt)
        food_type = self.get_food_type()
        food_quantity = get_food_quantity(quantity_prompt)
        stock_date = get_stock_date(stock_date_prompt)
        expiry_date = get_expiry_date(expiry_date_prompt)
        storage_type = self.get_storage_type()

        try:
            result = self.handler.handle_command(
                command="add",
                name=name,
                stock_date=stock_date,
                food_type=food_type,
                storage_type=storage_type,
                quantity=food_quantity,
                expiry_date=expiry_date
            )
            print(result)
        except Exception as e:
            print(f"Error adding food: {e}")

    def view_all_food(self):
        try:
            result = self.handler.view_all_food()
            print(result)
        except Exception as e:
            print(f"Error viewing food: {e}")

    def search_food(self, query:str, field:str):
        try:
            result = self.handler.advanced_search(query, field)
            print(result)
        except Exception as e:
            print(f"Error searching food: {e}")

def get_food_name(system_prompt:str, assertion_prompt:str):
    history = [{"role": "system", "content": system_prompt}]
    name = None

    while True:
        init_prompt = "What food do you want me to add?"
        history.append({"role": "system", "content": init_prompt})
        user_input = input(init_prompt)
        history.append({"role": "user", "content": user_input})
        call_1 = client.responses.create(
            model="gpt-4o-mini-2024-07-18",
            input=history,
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
            name = prop
            break
        else:
            print("That doesn't seem like a valid food. Let's try again.\n")

    return name


def get_food_quantity(system_prompt:str):
    pattern = r"\d+(l|g)"
    history = [{"role": "system", "content": system_prompt}]
    prop = ""
    while not re.match(pattern, prop):
        user_input = input("How much did you get?")
        history.append({"role": "user", "content": user_input})
        response = client.responses.create(
            model="gpt-4o-mini-2024-07-18",
            input=history,
            store=False
        )
        prop = response.output_text.strip().lower()
        history.append({"role": "assistant", "content": response.output_text.strip()})
    output = response.output_text.strip().lower()
    match = re.match(pattern, output)
    if match:
        return match.group()
    
def get_stock_date(system_prompt:str):
    pattern = r"^(today|yesterday|\d{2}-\d{2}-\d{4})$"
    history = [{"role": "system", "content": system_prompt}]
    prop = ""
    while not re.match(pattern, prop):
        user_input = input("When did you get this food? (e.g. today, yesterday, 24-05-2025): ")
        history.append({"role": "user", "content": user_input})
        response = client.responses.create(
            model="gpt-4o-mini-2024-07-18",
            input=history,
            store=False
        )
        prop = response.output_text.strip().lower()
        history.append({"role": "assistant", "content": response.output_text.strip()})
    output = response.output_text.strip().lower()
    match = re.match(pattern, output)
    if match:
        return match.group()
    
def get_expiry_date(system_prompt:str):
    pattern = r"^(none|\d{2}-\d{2}-\d{4})$"
    history = [{"role": "system", "content": system_prompt}]
    prop = ""
    while not re.match(pattern, prop):
        user_input = input("Is there an expiry date for this food? If yes, enter the date (e.g. 24-05-2025). Otherwise, enter 'none': ")
        history.append({"role": "user", "content": user_input})
        response = client.responses.create(
            model="gpt-4o-mini-2024-07-18",
            input=history,
            store=False
        )
        prop = response.output_text.strip().lower()
        history.append({"role": "assistant", "content": response.output_text.strip()})
    output = response.output_text.strip().lower()
    match = re.match(pattern, output)
    if match:
        return match.group()

# Create an instance of add_food
add_new_food = add_food()

if __name__ == "__main__":
    # Main loop
    while True:
        funnel = add_new_food.get_funnel()
        print(funnel)
        
        if funnel == "add":
            add_new_food.add_new_food()
        elif funnel == "view all":
            add_new_food.view_all_food()
        elif funnel == "search":
            add_new_food.search_food()
        elif funnel == "exit":
            print("Goodbye!")
            break

# user_prompt = input("What would you like to do today?")
# completion = client.beta.chat.completions.parse(
#     model="gpt-4o-mini",
#     messages=[
#         {"role": "system", "content": system_prompt},
#         {
#             "role": "user", 
#             "content": user_prompt
#         }
#     ],
#     response_format=RequestType
# )

# response = completion.choices[0].message.parsed
# print(response.request)
    
    # def ask_food_type(self):
    #     food_types = ["fruit", "vegetable", "edible oil", "dairy", "non-dairy", "breakfast-cereal", "meat", "grains", "beverage"]
        
    #     food_type = ""
    #     base_prompt = "What type of food is it?"
    #     persistent_querying()
        # while food_type not in food_types:
        #     type_prompt = input(base_prompt)
        #     completion = client.beta.chat.completions.parse(
        #         model="gpt-4o-mini",
        #         messages=[
        #             {"role": "system", "content": food_type_prompt},
        #             {
        #                 "role": "user", 
        #                 "content": type_prompt
        #             }
        #         ],
        #         response_format=TypeRequest
        #     )
        #     pre_res = completion.choices[0].message.parsed
        #     food_type = pre_res.food_type
        #     food_type = food_type.lower()
        #     base_prompt = pre_res.food_type
            
        # response = completion.choices[0].message.parsed
        # print(response.food_type)

# completion = client.beta.chat.completions.parse(
#             model="gpt-4o-mini",
#             messages=[
#                 {"role": "system", "content": system_prompt},
#                 {
#                     "role": "user", 
#                     "content": user_prompt
#                 }
#             ]
#         )
#         prop = completion.choices[0].message.content.strip().lower()
# response = completion.choices[0].message.content.strip().lower()

# {"role": "assistant", "content": "base_prompt"}