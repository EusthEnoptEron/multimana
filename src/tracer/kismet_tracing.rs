use std::ffi::c_void;
use manasdk::{EPropertyFlags, FFrame, UObject};
use manasdk::core_u_object::UFunction;
use crate::tracer::{EX_END_FUNCTION_PARAMS, GNATIVES};
use crate::tracer::to_string::{to_string_fproperty};

pub fn get_params(
    function: Option<&UFunction>,
    code_offset: usize,
    context: &UObject,
    stack: &FFrame,
) -> Option<String> {
    let mut params = String::new();
    let mut new_stack = stack.clone();

    if let Some(function) = function {
        if function.num_parms > 0
            && !function.child_properties().nth(0)?.property_flags.contains(EPropertyFlags::ReturnParm)
        {
            let save_code = new_stack.code;

            // Allocate memory for the frame
            let frame = vec![0u8; function.size as usize];

            // Step over the function pointer
            new_stack.code = new_stack.code.wrapping_add(code_offset);

            for property in function.child_properties() {
                if unsafe { *(new_stack.code) } == EX_END_FUNCTION_PARAMS {
                    break;
                }

                new_stack.most_recent_property_address = std::ptr::null_mut();

                // Skip the return parameter case, as we've already handled it above
                let is_return_param = property
                    .property_flags
                    .contains(EPropertyFlags::ReturnParm);
                if is_return_param {
                    continue;
                }

                let b = unsafe { *new_stack.code };
                new_stack.code = unsafe { new_stack.code.wrapping_add(1usize) };

                if property.property_flags.contains(EPropertyFlags::OutParm) {
                    // Log::info("Out {:p}", b);
                    GNATIVES.get()?[b as usize](
                        new_stack.object.into(),
                        &new_stack,
                        std::ptr::null_mut(),
                    );

                    if property.property_flags.contains(EPropertyFlags::ReferenceParm)
                    {
                        if let Some(most_recent_property_address) =
                            unsafe { new_stack.most_recent_property_address.as_ref() }
                        {
                            params.push_str(
                                to_string_fproperty(property, most_recent_property_address).as_str(),
                            );
                            params.push_str(", ");
                        }
                    }
                } else {
                    let addr = unsafe {
                        frame.as_ptr().add(property.offset as usize) as *mut c_void
                    };

                    if property
                        .property_flags
                        .contains(EPropertyFlags::ZeroConstructor)
                    {
                        unsafe {
                            std::ptr::write_bytes(
                                addr,
                                0,
                                ((*property).array_dim * (*property).element_size) as usize,
                            );
                        }
                    } else {
                        unsafe {
                            std::ptr::write_bytes(
                                addr,
                                0,
                                ((*property).array_dim * (*property).element_size) as usize,
                            );
                        }
                    }

                    // Log::info("b:{:p} ({}) ({:p}) ({})", addr, (*property).array_dim * (*property).element_size, MultiplayerMod::GNATIVES[b as usize], (*property).get_name());
                    GNATIVES.get()?[b as usize](new_stack.object.into(), &new_stack, addr);
                    params.push_str(to_string_fproperty(property, addr).as_str());
                    params.push_str(", ");
                    // Log::info("a:ok");
                }
            }

            new_stack.code = save_code;
        }
    }

    if params.ends_with(", ") {
        params.truncate(params.len() - 2);
    }

    Some(params)
}