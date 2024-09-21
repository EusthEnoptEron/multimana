import inspect
import unreal_engine as ue

def search_stack_for_parameter(predicate, max_depth = 1):
    """
    Traverses the call stack to find a parameter that satisfies the given predicate.

    Args:
        predicate (callable): A function that takes one argument and returns True if the condition is met.

    Returns:
        The first parameter that satisfies the predicate, or None if no such parameter is found.
        :param max_depth:
    """
    frame = inspect.currentframe()
    i = 0
    try:
        while frame:
            if i >= max_depth:
                break

            i += 1
            # Get local variables in the current frame
            local_vars = frame.f_locals
            # Iterate over local variables to find one that satisfies the predicate
            for var_name, var_value in local_vars.items():
                if predicate(var_value):
                    return var_value
            # Move to the previous frame in the call stack
            frame = frame.f_back
    except Exception as e:
        ue.log_error(f"{e}")
    finally:
        # Clean up to prevent reference cycles
        del frame
    return None  # Return None if no parameter satisfies the predicate