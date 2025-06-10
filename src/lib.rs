#![allow(unused_variables)]
#![allow(unused_assignments)]

// Standard library imports
use std::env;
use std::ffi::OsString;
use std::os::windows::ffi::OsStrExt;

// Windows Registry
use winreg::enums::*;
use winreg::RegKey;

// Windows API - COM
use winapi::um::combaseapi::{
    CoInitializeEx,
    CoInitializeSecurity,
    CoCreateInstance,
    CoUninitialize,
    CLSCTX_ALL,
};
use winapi::um::objbase::COINIT_MULTITHREADED;
use winapi::um::oleauto::VariantInit;
use winapi::um::oaidl::VARIANT;

// Windows API - RPC
use winapi::shared::rpcdce::{
    RPC_C_AUTHN_LEVEL_PKT_PRIVACY,
    RPC_C_IMP_LEVEL_IMPERSONATE,
};

// Windows API - GUID
use winapi::shared::guiddef::{
    IID,
    CLSID,
    GUID,
};
use winapi::ctypes::c_void as c_void_winapi;

// Windows API - Task Scheduler
use winapi::um::taskschd::{
    ITaskService,
    ITaskFolder,
    ITaskDefinition,
    IRegistrationInfo,
    ITaskSettings,
    ITriggerCollection,
    ITrigger,
    TASK_TRIGGER_LOGON,
    ILogonTrigger,
    IRegisteredTask,
};

// https://learn.microsoft.com/en-us/windows/win32/taskschd/logon-trigger-example--c---

// Define the CLSID for TaskScheduler
//these are hardcoded values I used during testing. The code should now find these values dynamically.
/* 
const CLSID_TaskScheduler: CLSID = CLSID {
    Data1: 0x0F87369F,
    Data2: 0xA4E5,
    Data3: 0x4CFC,
    Data4: [0xBD, 0x3E, 0x73, 0xE6, 0x15, 0x45, 0x72, 0xDD],
};

// Define the IID for ITaskService
//will probably need to find this dynamically to run on other machines
const IID_ITaskService: IID = IID {
    Data1: 0x2FABA4C7,
    Data2: 0x4DA9,
    Data3: 0x4013,
    Data4: [0x96, 0x97, 0x20, 0xCC, 0x3F, 0xD4, 0x0F, 0x85],
};
*/
/* 
// Define the IID for ILogonTrigger
const IID_ILogonTrigger: IID = IID {
    Data1: 0x72E9,
    Data2: 0x4E4D,
    Data3: 0x4F1A,
    Data4: [0x9C, 0x3C, 0x4D, 0x7D, 0x91, 0x9D, 0x1F, 0x5F],
};
*/
pub fn create_task(task_name: &str, task_path: &str, arguments: Option<&str>) -> String {
    // Try to initialize COM with multithreaded apartment, but don't fail if it's already initialized
    let result = unsafe { CoInitializeEx(
        std::ptr::null_mut(),
        COINIT_MULTITHREADED
    )};
    
    // S_FALSE (0x00000001) means COM was already initialized
    if result != 0 && result != 1 {
        return format!("Failed to initialize COM: {:x}", result);
    }

    // Try to initialize security, but don't fail if it's already initialized
    let result = unsafe { CoInitializeSecurity(
        std::ptr::null_mut(),  // pSecDesc
        -1,                    // cAuthSvc
        std::ptr::null_mut(),  // asAuthSvc
        std::ptr::null_mut(),  // pReserved1
        RPC_C_AUTHN_LEVEL_PKT_PRIVACY,  // dwAuthnLevel
        RPC_C_IMP_LEVEL_IMPERSONATE,    // dwImpLevel
        std::ptr::null_mut(),  // pAuthList
        0,                     // dwCapabilities
        std::ptr::null_mut(),  // pReserved3
    )};

    // RPC_E_TOO_LATE (0x80010119) means security was already initialized
    if result != 0 && result != -2147417831i32 {
        unsafe { CoUninitialize() };
        return format!("Failed to initialize COM security: {:x}", result);
    }

    // Find the CLSID and IID dynamically
    let (clsid, iid) = match find_task_scheduler_guids() {
        Ok(guids) => guids,
        Err(e) => {
            unsafe { CoUninitialize() };
            return format!("Failed to find Task Scheduler GUIDs: {}", e);
        }
    };

    // Convert task name to wide string (UTF-16)
    let task_name_wide: Vec<u16> = OsString::from(task_name)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    // Convert task path to wide string
    let task_path_wide: Vec<u16> = OsString::from(task_path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    // Create an instance of the Task Service
    let mut p_service: *mut ITaskService = std::ptr::null_mut();
    let hr = unsafe {
        CoCreateInstance(
            &clsid,
            std::ptr::null_mut(),
            CLSCTX_ALL,
            &iid,
            &mut p_service as *mut *mut ITaskService as *mut *mut c_void_winapi
        )
    };

    if hr != 0 {
        unsafe { CoUninitialize() };
        return format!("Failed to create an instance of ITaskService: {:x}", hr);
    }

    // Connect to the task service
    let mut empty_variant: VARIANT = unsafe { std::mem::zeroed() };
    unsafe { VariantInit(&mut empty_variant) };

    let hr = unsafe {
        (*p_service).Connect(
            empty_variant,  // serverName
            empty_variant,  // user
            empty_variant,  // domain
            empty_variant   // password
        )
    };

    if hr != 0 {
        unsafe {
            (*p_service).Release();
            CoUninitialize();
        }
        return format!("ITaskService::Connect failed: {:x}", hr);
    }

    // Get the root task folder
    let mut p_root_folder: *mut ITaskFolder = std::ptr::null_mut();
    let root_path: Vec<u16> = OsString::from("\\")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let hr = unsafe {
        (*p_service).GetFolder(
            root_path.as_ptr() as *mut u16,
            &mut p_root_folder
        )
    };

    if hr != 0 {
        unsafe {
            (*p_service).Release();
            CoUninitialize();
        }
        return format!("Cannot get Root Folder pointer: {:x}", hr);
    }

    // If the same task exists, remove it
    unsafe {
        (*p_root_folder).DeleteTask(
            task_name_wide.as_ptr() as *mut u16,
            0
        );
    }

    // Create the task builder object to create the task
    let mut p_task: *mut ITaskDefinition = std::ptr::null_mut();
    let hr = unsafe {
        (*p_service).NewTask(
            0,
            &mut p_task
        )
    };

    // Release p_service as it's no longer needed
    unsafe { (*p_service).Release() };

    if hr != 0 {
        unsafe {
            (*p_root_folder).Release();
            (*p_task).Release();
            CoUninitialize();
        }
        return format!("Failed to create a task definition: {:x}", hr);
    }

    // Get the registration info for setting the identification
    let mut p_reg_info: *mut IRegistrationInfo = std::ptr::null_mut();
    let hr = unsafe {
        (*p_task).get_RegistrationInfo(&mut p_reg_info)
    };

    if hr != 0 {
        unsafe {
            (*p_root_folder).Release();
            (*p_task).Release();
            CoUninitialize();
        }
        return format!("Cannot get identification pointer: {:x}", hr);
    }

    // Set the author name
    let author_name: Vec<u16> = OsString::from("Author Name")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let hr = unsafe {
        (*p_reg_info).put_Author(author_name.as_ptr() as *mut u16)
    };

    // Release the registration info as we're done with it
    unsafe { (*p_reg_info).Release() };

    if hr != 0 {
        unsafe {
            (*p_root_folder).Release();
            (*p_task).Release();
            CoUninitialize();
        }
        return format!("Cannot put identification info: {:x}", hr);
    }

    // Create the settings for the task
    let mut p_settings: *mut ITaskSettings = std::ptr::null_mut();
    let hr = unsafe {
        (*p_task).get_Settings(&mut p_settings)
    };

    if hr != 0 {
        unsafe {
            (*p_root_folder).Release();
            (*p_task).Release();
            CoUninitialize();
        }
        return format!("Cannot get settings pointer: {:x}", hr);
    }

    // Set setting values for the task
    let hr = unsafe {
        (*p_settings).put_StartWhenAvailable(winapi::shared::wtypes::VARIANT_TRUE)
    };

    // Release the settings as we're done with it
    unsafe { (*p_settings).Release() };

    if hr != 0 {
        unsafe {
            (*p_root_folder).Release();
            (*p_task).Release();
            CoUninitialize();
        }
        return format!("Cannot put setting info: {:x}", hr);
    }

    // Get the trigger collection to insert the logon trigger
    let mut p_trigger_collection: *mut ITriggerCollection = std::ptr::null_mut();
    let hr = unsafe {
        (*p_task).get_Triggers(&mut p_trigger_collection)
    };

    if hr != 0 {
        unsafe {
            (*p_root_folder).Release();
            (*p_task).Release();
            CoUninitialize();
        }
        return format!("Cannot get trigger collection: {:x}", hr);
    }

    // Add the logon trigger to the task
    let mut p_trigger: *mut ITrigger = std::ptr::null_mut();
    let hr = unsafe {
        (*p_trigger_collection).Create(TASK_TRIGGER_LOGON, &mut p_trigger)
    };

    // Release the trigger collection as we're done with it
    unsafe { (*p_trigger_collection).Release() };

    if hr != 0 {
        unsafe {
            (*p_root_folder).Release();
            (*p_task).Release();
            CoUninitialize();
        }
        return format!("Cannot create the trigger: {:x}", hr);
    }

    // Get the logon trigger interface
    let mut p_logon_trigger: *mut ILogonTrigger = std::ptr::null_mut();
    
    // Try to get the interface directly from the trigger object
    let hr = unsafe {
        (*p_trigger).QueryInterface(
            &<ILogonTrigger as winapi::Interface>::uuidof(),
            &mut p_logon_trigger as *mut *mut ILogonTrigger as *mut *mut c_void_winapi
        )
    };

    // Release the trigger as we're done with it
    unsafe { (*p_trigger).Release() };

    if hr != 0 {
        unsafe {
            (*p_root_folder).Release();
            (*p_task).Release();
            CoUninitialize();
        }
        return format!("QueryInterface call failed for ILogonTrigger: {:x}", hr);
    }

    // Set the trigger ID
    let trigger_id: Vec<u16> = OsString::from("Trigger1")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    
    let hr = unsafe {
        (*p_logon_trigger).put_Id(trigger_id.as_ptr() as *mut u16)
    };

    if hr != 0 {
        unsafe {
            (*p_root_folder).Release();
            (*p_task).Release();
            (*p_logon_trigger).Release();
            CoUninitialize();
        }
        return format!("Cannot put the trigger ID: {:x}", hr);
    }

    // Set the start boundary to yesterday
    let start_boundary: Vec<u16> = OsString::from("2024-03-19T00:00:00")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    
    let hr = unsafe {
        (*p_logon_trigger).put_StartBoundary(start_boundary.as_ptr() as *mut u16)
    };

    if hr != 0 {
        unsafe {
            (*p_root_folder).Release();
            (*p_task).Release();
            (*p_logon_trigger).Release();
            CoUninitialize();
        }
        return format!("Cannot put the start boundary: {:x}", hr);
    }

    // Set the end boundary to one year from now
    let end_boundary: Vec<u16> = OsString::from("2026-06-06T00:00:00")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    
    let hr = unsafe {
        (*p_logon_trigger).put_EndBoundary(end_boundary.as_ptr() as *mut u16)
    };

    if hr != 0 {
        unsafe {
            (*p_root_folder).Release();
            (*p_task).Release();
            (*p_logon_trigger).Release();
            CoUninitialize();
        }
        return format!("Cannot put the end boundary: {:x}", hr);
    }

    // Get the current user's domain and username
    let user_id = format!("{}\\{}", 
        env::var("USERDOMAIN").unwrap_or_else(|_| ".".to_string()),
        env::var("USERNAME").unwrap_or_else(|_| "SYSTEM".to_string())
    );
    
    let user_id_wide: Vec<u16> = OsString::from(user_id)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let hr = unsafe {
        (*p_logon_trigger).put_UserId(user_id_wide.as_ptr() as *mut u16)
    };

    // Release the logon trigger as we're done with it
    unsafe { (*p_logon_trigger).Release() };

    if hr != 0 {
        unsafe {
            (*p_root_folder).Release();
            (*p_task).Release();
            CoUninitialize();
        }
        return format!("Cannot add user ID to logon trigger: {:x}", hr);
    }

    // Get the task action collection
    let mut p_action_collection: *mut winapi::um::taskschd::IActionCollection = std::ptr::null_mut();
    let hr = unsafe {
        (*p_task).get_Actions(&mut p_action_collection)
    };

    if hr != 0 {
        unsafe {
            (*p_root_folder).Release();
            (*p_task).Release();
            CoUninitialize();
        }
        return format!("Cannot get Task collection pointer: {:x}", hr);
    }

    // Create the action, specifying that it is an executable action
    let mut p_action: *mut winapi::um::taskschd::IAction = std::ptr::null_mut();
    let hr = unsafe {
        (*p_action_collection).Create(
            winapi::um::taskschd::TASK_ACTION_EXEC,
            &mut p_action
        )
    };

    // Release the action collection as we're done with it
    unsafe { (*p_action_collection).Release() };

    if hr != 0 {
        unsafe {
            (*p_root_folder).Release();
            (*p_task).Release();
            CoUninitialize();
        }
        return format!("Cannot create the action: {:x}", hr);
    }

    // Get the executable action interface
    let mut p_exec_action: *mut winapi::um::taskschd::IExecAction = std::ptr::null_mut();
    
    // Query for the executable task pointer
    let hr = unsafe {
        (*p_action).QueryInterface(
            &<winapi::um::taskschd::IExecAction as winapi::Interface>::uuidof(),
            &mut p_exec_action as *mut *mut winapi::um::taskschd::IExecAction as *mut *mut c_void_winapi
        )
    };

    // Release the action as we're done with it
    unsafe { (*p_action).Release() };

    if hr != 0 {
        unsafe {
            (*p_root_folder).Release();
            (*p_task).Release();
            CoUninitialize();
        }
        return format!("QueryInterface call failed for IExecAction: {:x}", hr);
    }

    // Set the path of the executable
    let hr = unsafe {
        (*p_exec_action).put_Path(task_path_wide.as_ptr() as *mut u16)
    };

    if hr != 0 {
        unsafe {
            (*p_root_folder).Release();
            (*p_task).Release();
            CoUninitialize();
        }
        return format!("Cannot set path of executable: {:x}", hr);
    }

    // Set arguments if provided
    if let Some(args) = arguments {
        let args_wide: Vec<u16> = OsString::from(args)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let hr = unsafe {
            (*p_exec_action).put_Arguments(args_wide.as_ptr() as *mut u16)
        };

        if hr != 0 {
            unsafe {
                (*p_root_folder).Release();
                (*p_task).Release();
                CoUninitialize();
            }
            return format!("Cannot set arguments: {:x}", hr);
        }
    }

    // Release the executable action as we're done with it
    unsafe { (*p_exec_action).Release() };

    // Register the task
    let mut p_registered_task: *mut IRegisteredTask = std::ptr::null_mut();
    
    // Create empty variants for optional parameters
    let mut empty_variant: VARIANT = unsafe { std::mem::zeroed() };
    unsafe { VariantInit(&mut empty_variant) };

    // Create variant for the current user (empty string)
    let mut user_variant: VARIANT = unsafe { std::mem::zeroed() };
    unsafe { VariantInit(&mut user_variant) };

    // Create empty string variant
    let mut empty_str_variant: VARIANT = unsafe { std::mem::zeroed() };
    unsafe { VariantInit(&mut empty_str_variant) };

    let hr = unsafe {
        (*p_root_folder).RegisterTaskDefinition(
            task_name_wide.as_ptr() as *mut u16,
            p_task,
            winapi::um::taskschd::TASK_CREATE_OR_UPDATE as i32,
            user_variant,  // Empty string for current user
            empty_variant, // Empty password
            winapi::um::taskschd::TASK_LOGON_INTERACTIVE_TOKEN,  // Use interactive token for current user
            empty_str_variant,
            &mut p_registered_task
        )
    };

    if hr != 0 {
        unsafe {
            (*p_root_folder).Release();
            (*p_task).Release();
            CoUninitialize();
        }
        return format!("Error saving the Task: {:x}", hr);
    }

    // Clean up
    unsafe {
        if !p_registered_task.is_null() {
            (*p_registered_task).Release();
        }
        if !p_root_folder.is_null() {
            (*p_root_folder).Release();
        }
        if !p_task.is_null() {
            (*p_task).Release();
        }
        CoUninitialize();
    }

    "Task successfully created".to_string()
}

fn parse_guid(guid_str: &str) -> Result<GUID, String> {
    // Remove curly braces if present
    let guid_str = guid_str.trim_matches(|c| c == '{' || c == '}');
    
    // Split into parts
    let parts: Vec<&str> = guid_str.split('-').collect();
    if parts.len() != 5 {
        return Err("Invalid GUID format".to_string());
    }

    // Parse Data1 (first part)
    let data1 = u32::from_str_radix(parts[0], 16)
        .map_err(|_| "Failed to parse Data1".to_string())?;

    // Parse Data2 (second part)
    let data2 = u16::from_str_radix(parts[1], 16)
        .map_err(|_| "Failed to parse Data2".to_string())?;

    // Parse Data3 (third part)
    let data3 = u16::from_str_radix(parts[2], 16)
        .map_err(|_| "Failed to parse Data3".to_string())?;

    // Parse Data4 (last two parts combined)
    let data4_1 = u8::from_str_radix(&parts[3][0..2], 16)
        .map_err(|_| "Failed to parse Data4[0]".to_string())?;
    let data4_2 = u8::from_str_radix(&parts[3][2..4], 16)
        .map_err(|_| "Failed to parse Data4[1]".to_string())?;
    
    // Parse the last part (6 bytes)
    let last_part = parts[4];
    if last_part.len() != 12 {
        return Err("Invalid Data4 format".to_string());
    }

    let data4_rest: Vec<u8> = (0..6)
        .map(|i| {
            let start = i * 2;
            let end = start + 2;
            u8::from_str_radix(&last_part[start..end], 16)
                .unwrap_or(0)
        })
        .collect();

    Ok(GUID {
        Data1: data1,
        Data2: data2,
        Data3: data3,
        Data4: [
            data4_1,
            data4_2,
            data4_rest[0],
            data4_rest[1],
            data4_rest[2],
            data4_rest[3],
            data4_rest[4],
            data4_rest[5],
        ],
    })
}
/* */
fn find_task_scheduler_guids() -> Result<(CLSID, IID), String> {
    // Open the CLSID key
    let hkcr = RegKey::predef(HKEY_CLASSES_ROOT);
    let clsid_key = hkcr.open_subkey("CLSID").map_err(|e| e.to_string())?;

    // Search for the TaskScheduler class
    let mut task_scheduler_clsid = None;
    for subkey_name in clsid_key.enum_keys().map(|x| x.unwrap()) {
        let subkey = clsid_key.open_subkey(&subkey_name).map_err(|e| e.to_string())?;
        let default_value: String = subkey.get_value("").unwrap_or_default();
        if default_value.contains("TaskScheduler") {
            task_scheduler_clsid = Some(subkey_name);
            break;
        }
    }

    let clsid = task_scheduler_clsid.ok_or("TaskScheduler CLSID not found")?;

    // Open the Interface key
    let interface_key = hkcr.open_subkey("Interface").map_err(|e| e.to_string())?;

    // Search for the ITaskService interface
    let mut task_service_iid = None;
    for subkey_name in interface_key.enum_keys().map(|x| x.unwrap()) {
        let subkey = interface_key.open_subkey(&subkey_name).map_err(|e| e.to_string())?;
        let default_value: String = subkey.get_value("").unwrap_or_default();
        if default_value.contains("ITaskService") {
            task_service_iid = Some(subkey_name);
            break;
        }
    }

    let iid = task_service_iid.ok_or("ITaskService IID not found")?;

    // Convert the CLSID and IID strings to GUID structs
    let clsid_guid = parse_guid(&clsid)?;
    let iid_guid = parse_guid(&iid)?;

    Ok((clsid_guid, iid_guid))
}