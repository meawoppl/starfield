#!/usr/bin/env python
"""
Test file for the format_py_error function in python_comparison.rs
This file generates different types of Python exceptions that can be 
used to test the error formatting functionality in Rust.
"""
import sys
import json
import traceback


def generate_error_scenarios():
    """Generate different error scenarios and return their serialized info"""
    scenarios = []
    
    try:
        # Scenario 1: Simple ValueError
        try:
            raise ValueError("This is a test error")
        except Exception as e:
            scenarios.append({
                "name": "simple_value_error",
                "error_type": type(e).__name__,
                "message": str(e),
                "traceback": traceback.format_exc()
            })
        
        # Scenario 2: IndexError with traceback
        try:
            def function_with_error():
                my_list = [1, 2, 3]
                return my_list[10]  # Will cause IndexError
            
            function_with_error()
        except Exception as e:
            scenarios.append({
                "name": "index_error_with_traceback",
                "error_type": type(e).__name__,
                "message": str(e),
                "traceback": traceback.format_exc()
            })
        
        # Scenario 3: Custom exception
        try:
            class CustomError(Exception):
                pass
            
            raise CustomError("This is a custom error")
        except Exception as e:
            scenarios.append({
                "name": "custom_exception",
                "error_type": type(e).__name__,
                "message": str(e),
                "traceback": traceback.format_exc()
            })
        
        # Scenario 4: Nested exception
        try:
            def outer_function():
                def inner_function():
                    x = 1 / 0  # ZeroDivisionError
                    return x
                return inner_function()
            
            outer_function()
        except Exception as e:
            scenarios.append({
                "name": "nested_exception",
                "error_type": type(e).__name__,
                "message": str(e),
                "traceback": traceback.format_exc()
            })
            
        # Scenario 5: Name error (undefined variable)
        try:
            print(undefined_variable)
        except Exception as e:
            scenarios.append({
                "name": "name_error",
                "error_type": type(e).__name__,
                "message": str(e),
                "traceback": traceback.format_exc()
            })
        
        # Return successfully generated scenarios
        return {"success": True, "scenarios": scenarios}
        
    except Exception as outer_e:
        # Meta error - something went wrong while generating the test scenarios
        return {
            "success": False,
            "error": str(outer_e),
            "error_type": type(outer_e).__name__,
            "traceback": traceback.format_exc()
        }


# Execute the function and print results as JSON
if __name__ == "__main__":
    result = generate_error_scenarios()
    print(json.dumps(result))