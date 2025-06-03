tools = [{
    "type": "function",
    "function": {
        "name": "advanced_search",
        "description": "Performs an advanced search on food stock data based on a keyword and field.",
        "strict": True,
        "parameters": {
            "type": "object",
            "required": [
                "keyword",
                "field"
            ],
            "properties": {
                "keyword": {
                    "type": "string",
                    "description": "The search keyword used for querying food stock."
                },
                "field": {
                    "type": "string",
                    "description": "The field in the food stock data against which to perform the search. Allowed values: 'name', 'food_type', 'nutrient', 'storage_type'."
                }
            },
            "additionalProperties": False
        }
    }
}, 
{
    "type": "function",
    "function": {
        "name": "get_all_food",
        "description": "Retrieves all food stock information from the database",
        "strict": True,
        "parameters": {
            "type": "object",
            "properties": {},
            "additionalProperties": False
        }
    }
},
{
    "type": "function",
    "function": {
        "name": "collect_food_info",
        "description": "Starts an interactive process to collect detailed information about food items from the user. Use this when the user wants to add food to their inventory.",
        "strict": True,
        "parameters": {
            "type": "object",
            "properties": {},
            "additionalProperties": False
        }
    }
}]