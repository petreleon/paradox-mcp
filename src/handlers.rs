use crate::args::Args;
use crate::mcp::RpcRequest;
use crate::pxlib;
use serde_json::{json, Map, Value};
use std::ffi::CString;
use std::path::Path;

pub fn handle_request(req: &RpcRequest, args: &Args) -> Value {
    match req.method.as_str() {
        "initialize" => {
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "paradox-mcp-rust",
                    "version": "1.0.0"
                }
            })
        }
        "tools/list" => {
            json!({
                "tools": [
                    {
                        "name": "get_server_status",
                        "description": "Get the status and configuration of the Paradox MCP server",
                        "inputSchema": {
                            "type": "object",
                            "properties": {}
                        }
                    },
                    {
                        "name": "list_tables",
                        "description": "List all Paradox tables (.db files) in the configured location",
                        "inputSchema": {
                            "type": "object",
                            "properties": {}
                        }
                    },
                    {
                        "name": "read_table_schema",
                        "description": "Read the schema (field names and types) of a Paradox table",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "table_name": {
                                    "type": "string",
                                    "description": "The name of the table (e.g., 'customers')"
                                }
                            },
                            "required": ["table_name"]
                        }
                    },
                    {
                        "name": "read_table_data",
                        "description": "Read records from a Paradox table",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "table_name": {
                                    "type": "string",
                                    "description": "The name of the table"
                                },
                                "limit": {
                                    "type": "integer",
                                    "description": "Maximum number of records to read (default: 100)",
                                    "default": 100
                                }
                            },
                            "required": ["table_name"]
                        }
                    },
                    {
                        "name": "search_table",
                        "description": "Search for specific records in a Paradox table by field values",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "table_name": {
                                    "type": "string",
                                    "description": "The name of the table"
                                },
                                "query": {
                                    "type": "object",
                                    "description": "Field-value pairs to match (e.g., {\"ID\": \"123\"})"
                                }
                            },
                            "required": ["table_name", "query"]
                        }
                    },
                    {
                        "name": "create_table",
                        "description": "Create a new Paradox table with a specific schema (requires editing permission)",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "table_name": {
                                    "type": "string",
                                    "description": "The name of the table to create (e.g., 'new_table')"
                                },
                                "fields": {
                                    "type": "array",
                                    "items": {
                                        "type": "object",
                                        "properties": {
                                            "name": { "type": "string", "description": "Field name" },
                                            "type": { "type": "string", "description": "Field type (ALPHA, SHORT, LONG, NUMBER, DATE, LOGICAL, etc.)" },
                                            "length": { "type": "integer", "description": "Length for ALPHA fields" }
                                        },
                                        "required": ["name", "type"]
                                    },
                                    "description": "Array of field definitions"
                                }
                            },
                            "required": ["table_name", "fields"]
                        }
                    },
                    {
                        "name": "insert_record",
                        "description": "Add a new record to a Paradox table (requires editing permission)",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "table_name": {
                                    "type": "string",
                                    "description": "The name of the table"
                                },
                                "record": {
                                    "type": "object",
                                    "description": "The record data to insert"
                                }
                            },
                            "required": ["table_name", "record"]
                        }
                    },
                    {
                        "name": "update_record",
                        "description": "Update an existing record in a Paradox table (requires editing permission)",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "table_name": {
                                    "type": "string",
                                    "description": "The name of the table"
                                },
                                "index": {
                                    "type": "integer",
                                    "description": "The 0-based index of the record to update"
                                },
                                "record": {
                                    "type": "object",
                                    "description": "The new record data"
                                }
                            },
                            "required": ["table_name", "index", "record"]
                        }
                    }
                ]
            })
        }
        "tools/call" => {
            if let Some(params) = &req.params {
                if let Some(name) = params.get("name").and_then(|n| n.as_str()) {
                    eprintln!("DEBUG: Handling tool call: {}", name);
                    let empty_map = Map::new();
                    let arguments = params
                        .get("arguments")
                        .and_then(|a| a.as_object())
                        .unwrap_or(&empty_map);

                    match name {
                        "get_server_status" => {
                            let text = format!("Paradox Server Configuration:\n- Location: {}\n- Permit Editing: {}", args.location, args.permit_editing);
                            json!({
                                "content": [{ "type": "text", "text": text }]
                            })
                        }
                        "list_tables" => handle_list_tables(args),
                        "read_table_schema" => {
                            if let Some(table_name) =
                                arguments.get("table_name").and_then(|t| t.as_str())
                            {
                                handle_read_schema(table_name, &args.location)
                            } else {
                                json!({ "isError": true, "content": [{ "type": "text", "text": "Missing table_name" }] })
                            }
                        }
                        "read_table_data" => {
                            if let Some(table_name) =
                                arguments.get("table_name").and_then(|t| t.as_str())
                            {
                                let limit = arguments
                                    .get("limit")
                                    .and_then(|l| l.as_u64())
                                    .unwrap_or(100)
                                    as i32;
                                handle_read_data(table_name, &args.location, limit)
                            } else {
                                json!({ "isError": true, "content": [{ "type": "text", "text": "Missing table_name" }] })
                            }
                        }
                        "search_table" => {
                            if let Some(table_name) =
                                arguments.get("table_name").and_then(|t| t.as_str())
                            {
                                if let Some(query) =
                                    arguments.get("query").and_then(|q| q.as_object())
                                {
                                    handle_search_table(table_name, &args.location, query)
                                } else {
                                    json!({ "isError": true, "content": [{ "type": "text", "text": "Missing or invalid query object" }] })
                                }
                            } else {
                                json!({ "isError": true, "content": [{ "type": "text", "text": "Missing table_name" }] })
                            }
                        }
                        "create_table" => {
                            if !args.permit_editing {
                                return json!({ "isError": true, "content": [{ "type": "text", "text": "Editing is not permitted on this server." }] });
                            }
                            if let Some(table_name) =
                                arguments.get("table_name").and_then(|t| t.as_str())
                            {
                                if let Some(fields) =
                                    arguments.get("fields").and_then(|f| f.as_array())
                                {
                                    handle_create_table(table_name, &args.location, fields)
                                } else {
                                    json!({ "isError": true, "content": [{ "type": "text", "text": "Missing or invalid fields array" }] })
                                }
                            } else {
                                json!({ "isError": true, "content": [{ "type": "text", "text": "Missing table_name" }] })
                            }
                        }
                        "insert_record" => {
                            if !args.permit_editing {
                                return json!({ "isError": true, "content": [{ "type": "text", "text": "Editing is not permitted on this server." }] });
                            }
                            if let Some(table_name) =
                                arguments.get("table_name").and_then(|t| t.as_str())
                            {
                                if let Some(record) =
                                    arguments.get("record").and_then(|r| r.as_object())
                                {
                                    handle_write_record(table_name, &args.location, None, record)
                                } else {
                                    json!({ "isError": true, "content": [{ "type": "text", "text": "Missing record object" }] })
                                }
                            } else {
                                json!({ "isError": true, "content": [{ "type": "text", "text": "Missing table_name" }] })
                            }
                        }
                        "update_record" => {
                            if !args.permit_editing {
                                return json!({ "isError": true, "content": [{ "type": "text", "text": "Editing is not permitted on this server." }] });
                            }
                            if let Some(table_name) =
                                arguments.get("table_name").and_then(|t| t.as_str())
                            {
                                let index = arguments
                                    .get("index")
                                    .and_then(|i| i.as_u64())
                                    .map(|i| i as i32);
                                if let Some(record) =
                                    arguments.get("record").and_then(|r| r.as_object())
                                {
                                    if let Some(idx) = index {
                                        handle_write_record(
                                            table_name,
                                            &args.location,
                                            Some(idx),
                                            record,
                                        )
                                    } else {
                                        json!({ "isError": true, "content": [{ "type": "text", "text": "Missing record index" }] })
                                    }
                                } else {
                                    json!({ "isError": true, "content": [{ "type": "text", "text": "Missing record object" }] })
                                }
                            } else {
                                json!({ "isError": true, "content": [{ "type": "text", "text": "Missing table_name" }] })
                            }
                        }
                        _ => {
                            json!({ "isError": true, "content": [{ "type": "text", "text": format!("Tool not found: {}", name) }] })
                        }
                    }
                } else {
                    json!({ "isError": true, "content": [{ "type": "text", "text": "Missing tool name" }] })
                }
            } else {
                json!({ "isError": true, "content": [{ "type": "text", "text": "Missing params" }] })
            }
        }
        _ => json!({}),
    }
}

fn handle_list_tables(args: &Args) -> Value {
    let mut tables = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&args.location) {
        for res in entries {
            if let Ok(entry) = res {
                if entry.path().extension().and_then(|o| o.to_str()) == Some("db") {
                    if let Some(name) = entry.path().file_name().and_then(|n| n.to_str()) {
                        tables.push(name.to_string());
                    }
                }
            }
        }
    }
    if tables.is_empty() {
        json!({
            "content": [{ "type": "text", "text": "No .db files found in location." }]
        })
    } else {
        json!({
            "content": [{ "type": "text", "text": format!("Found tables: {}", tables.join(", ")) }]
        })
    }
}

fn handle_read_schema(table_name: &str, location: &str) -> Value {
    let mut full_path = Path::new(location).join(table_name);
    if full_path.extension().is_none() {
        full_path.set_extension("db");
    }

    let path_str = full_path.to_string_lossy();

    unsafe {
        let pxdoc = pxlib::PX_new();
        if pxdoc.is_null() {
            return json!({ "isError": true, "content": [{ "type": "text", "text": "Failed to initialize PX library." }] });
        }

        let c_path = match CString::new(path_str.as_ref()) {
            Ok(c) => c,
            Err(_) => {
                pxlib::PX_delete(pxdoc);
                return json!({ "isError": true, "content": [{ "type": "text", "text": "Invalid table path string." }] });
            }
        };

        if pxlib::PX_open_file(pxdoc, c_path.as_ptr()) < 0 {
            pxlib::PX_delete(pxdoc);
            return json!({ "isError": true, "content": [{ "type": "text", "text": format!("Failed to open table '{}'", path_str) }] });
        }

        let num_fields = pxlib::PX_get_num_fields(pxdoc);
        let fields_ptr = pxlib::PX_get_fields(pxdoc);
        let mut fields_info = Vec::new();

        if !fields_ptr.is_null() {
            let fields_slice = std::slice::from_raw_parts(fields_ptr, num_fields as usize);
            for f in fields_slice {
                if !f.px_fname.is_null() {
                    let name = std::ffi::CStr::from_ptr(f.px_fname)
                        .to_string_lossy()
                        .into_owned();
                    let ftype = f.px_ftype;
                    let flen = f.px_flen;

                    let type_str = match ftype as u32 {
                        pxlib::pxfAlpha => "ALPHA",
                        pxlib::pxfDate => "DATE",
                        pxlib::pxfShort => "SHORT",
                        pxlib::pxfLong => "LONG",
                        pxlib::pxfCurrency => "CURRENCY",
                        pxlib::pxfNumber => "NUMBER",
                        pxlib::pxfLogical => "LOGICAL",
                        pxlib::pxfMemoBLOb => "MEMO",
                        pxlib::pxfBLOb => "BLOB",
                        pxlib::pxfTime => "TIME",
                        pxlib::pxfTimestamp => "TIMESTAMP",
                        pxlib::pxfAutoInc => "AUTOINC",
                        pxlib::pxfBCD => "BCD",
                        pxlib::pxfBytes => "BYTES",
                        _ => "UNKNOWN",
                    };

                    fields_info.push(json!({
                        "name": name,
                        "type": type_str,
                        "length": flen
                    }));
                }
            }
        }

        pxlib::PX_close(pxdoc);
        pxlib::PX_delete(pxdoc);

        json!({
            "content": [
                { "type": "text", "text": format!("Schema for table '{}':", table_name) },
                { "type": "text", "text": serde_json::to_string_pretty(&fields_info).unwrap() }
            ]
        })
    }
}

fn handle_read_data(table_name: &str, location: &str, limit: i32) -> Value {
    let mut full_path = Path::new(location).join(table_name);
    if full_path.extension().is_none() {
        full_path.set_extension("db");
    }

    let path_str = full_path.to_string_lossy();

    unsafe {
        let pxdoc = pxlib::PX_new();
        if pxdoc.is_null() {
            return json!({ "isError": true, "content": [{ "type": "text", "text": "Failed to initialize PX library." }] });
        }

        let c_path = match CString::new(path_str.as_ref()) {
            Ok(c) => c,
            Err(_) => {
                pxlib::PX_delete(pxdoc);
                return json!({ "isError": true, "content": [{ "type": "text", "text": "Invalid table path string." }] });
            }
        };

        if pxlib::PX_open_file(pxdoc, c_path.as_ptr()) < 0 {
            pxlib::PX_delete(pxdoc);
            return json!({ "isError": true, "content": [{ "type": "text", "text": format!("Failed to open table '{}'", path_str) }] });
        }

        let num_records = pxlib::PX_get_num_records(pxdoc);
        let num_fields = pxlib::PX_get_num_fields(pxdoc);
        let fields_ptr = pxlib::PX_get_fields(pxdoc);
        let fields_slice = std::slice::from_raw_parts(fields_ptr, num_fields as usize);

        let record_size = pxlib::PX_get_recordsize(pxdoc);
        let mut buf = vec![0u8; record_size as usize];
        let mut results = Vec::new();

        let count = if num_records < limit {
            num_records
        } else {
            limit
        };

        for i in 0..count {
            if !pxlib::PX_get_record(pxdoc, i, buf.as_mut_ptr()).is_null() {
                let mut record_map = Map::new();
                let mut offset = 0;
                for f_idx in 0..num_fields {
                    let f = &fields_slice[f_idx as usize];
                    let field_name = std::ffi::CStr::from_ptr(f.px_fname)
                        .to_string_lossy()
                        .into_owned();
                    let field_type = f.px_ftype;
                    let field_len = f.px_flen;

                    let val =
                        get_field_value(pxdoc, buf.as_mut_ptr().add(offset), field_type, field_len);
                    record_map.insert(field_name, val);

                    offset += field_len as usize;
                }
                results.push(Value::Object(record_map));
            }
        }

        pxlib::PX_close(pxdoc);
        pxlib::PX_delete(pxdoc);

        json!({
            "content": [
                { "type": "text", "text": format!("Data for table '{}' ({} records):", table_name, count) },
                { "type": "text", "text": serde_json::to_string_pretty(&results).unwrap() }
            ]
        })
    }
}

fn handle_search_table(table_name: &str, location: &str, query: &Map<String, Value>) -> Value {
    let mut full_path = Path::new(location).join(table_name);
    if full_path.extension().is_none() {
        full_path.set_extension("db");
    }

    let path_str = full_path.to_string_lossy();

    unsafe {
        let pxdoc = pxlib::PX_new();
        if pxdoc.is_null() {
            return json!({ "isError": true, "content": [{ "type": "text", "text": "Failed to initialize PX library." }] });
        }

        let c_path = match CString::new(path_str.as_ref()) {
            Ok(c) => c,
            Err(_) => {
                pxlib::PX_delete(pxdoc);
                return json!({ "isError": true, "content": [{ "type": "text", "text": "Invalid table path string." }] });
            }
        };

        if pxlib::PX_open_file(pxdoc, c_path.as_ptr()) < 0 {
            pxlib::PX_delete(pxdoc);
            return json!({ "isError": true, "content": [{ "type": "text", "text": format!("Failed to open table '{}'", path_str) }] });
        }

        let num_records = pxlib::PX_get_num_records(pxdoc);
        let num_fields = pxlib::PX_get_num_fields(pxdoc);
        let fields_ptr = pxlib::PX_get_fields(pxdoc);
        let fields_slice = std::slice::from_raw_parts(fields_ptr, num_fields as usize);

        let record_size = pxlib::PX_get_recordsize(pxdoc);
        let mut buf = vec![0u8; record_size as usize];
        let mut results = Vec::new();

        for i in 0..num_records {
            if !pxlib::PX_get_record(pxdoc, i, buf.as_mut_ptr()).is_null() {
                let mut record_map = Map::new();
                let mut matches = true;

                let mut offset = 0;
                for f_idx in 0..num_fields {
                    let f = &fields_slice[f_idx as usize];
                    let field_name = std::ffi::CStr::from_ptr(f.px_fname)
                        .to_string_lossy()
                        .into_owned();
                    let field_type = f.px_ftype;
                    let field_len = f.px_flen;

                    let val =
                        get_field_value(pxdoc, buf.as_mut_ptr().add(offset), field_type, field_len);

                    if let Some(query_val) = query.get(&field_name) {
                        if !compare_values(&val, query_val) {
                            matches = false;
                        }
                    }
                    record_map.insert(field_name, val);

                    offset += field_len as usize;
                }

                if matches {
                    results.push(Value::Object(record_map));
                }
            }
            if results.len() >= 1000 {
                break;
            } // Safety limit
        }

        pxlib::PX_close(pxdoc);
        pxlib::PX_delete(pxdoc);

        json!({
            "content": [
                { "type": "text", "text": format!("Search results for table '{}' ({} found):", table_name, results.len()) },
                { "type": "text", "text": serde_json::to_string_pretty(&results).unwrap() }
            ]
        })
    }
}

fn handle_create_table(table_name: &str, location: &str, fields: &Vec<Value>) -> Value {
    let mut full_path = Path::new(location).join(table_name);
    if full_path.extension().is_none() {
        full_path.set_extension("db");
    }

    let path_str = full_path.to_string_lossy();

    #[repr(C)]
    struct PxField {
        px_fname: *mut std::os::raw::c_char,
        px_ftype: std::os::raw::c_char,
        px_flen: std::os::raw::c_int,
        px_fdc: std::os::raw::c_int,
    }

    extern "C" {
        fn malloc(size: usize) -> *mut std::ffi::c_void;
        fn strdup(s: *const std::os::raw::c_char) -> *mut std::os::raw::c_char;
    }

    unsafe {
        let pxdoc = pxlib::PX_new();
        if pxdoc.is_null() {
            return json!({ "isError": true, "content": [{ "type": "text", "text": "Failed to initialize PX library." }] });
        }

        let fields_byte_size = std::mem::size_of::<PxField>() * fields.len();
        let px_fields_ptr = malloc(fields_byte_size) as *mut PxField;

        for (i, f_val) in fields.iter().enumerate() {
            let name_str = f_val
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("UNKNOWN");
            let type_str = f_val
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("ALPHA");
            let length = f_val.get("length").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

            // Allocate string using C's strdup so pxlib can free it
            let c_name =
                CString::new(name_str).unwrap_or_else(|_| CString::new("INVALID").unwrap());
            let c_name_ptr = strdup(c_name.as_ptr());

            let f_type = match type_str.to_uppercase().as_str() {
                "ALPHA" => pxlib::pxfAlpha,
                "DATE" => pxlib::pxfDate,
                "SHORT" => pxlib::pxfShort,
                "LONG" => pxlib::pxfLong,
                "CURRENCY" => pxlib::pxfCurrency,
                "NUMBER" => pxlib::pxfNumber,
                "LOGICAL" => pxlib::pxfLogical,
                "MEMO" => pxlib::pxfMemoBLOb,
                "BLOB" => pxlib::pxfBLOb,
                "TIME" => pxlib::pxfTime,
                "TIMESTAMP" => pxlib::pxfTimestamp,
                "AUTOINC" => pxlib::pxfAutoInc,
                "BCD" => pxlib::pxfBCD,
                "BYTES" => pxlib::pxfBytes,
                _ => pxlib::pxfAlpha,
            };

            let final_length = if length > 0 {
                length
            } else {
                match f_type as u32 {
                    pxlib::pxfShort => 2,
                    pxlib::pxfLong | pxlib::pxfAutoInc | pxlib::pxfDate | pxlib::pxfTime => 4,
                    pxlib::pxfCurrency | pxlib::pxfNumber | pxlib::pxfTimestamp => 8,
                    pxlib::pxfLogical => 1,
                    _ => 0,
                }
            };

            // Write into the malloc'd array directly to avoid double free
            std::ptr::write(
                px_fields_ptr.add(i),
                PxField {
                    px_fname: c_name_ptr,
                    px_ftype: f_type as std::os::raw::c_char,
                    px_flen: final_length,
                    px_fdc: 0,
                },
            );
        }

        let c_path = match CString::new(path_str.as_ref()) {
            Ok(c) => c,
            Err(_) => {
                pxlib::PX_delete(pxdoc);
                return json!({ "isError": true, "content": [{ "type": "text", "text": "Invalid table path string." }] });
            }
        };

        // File type 0 = pxfFileTypIndexDB
        let res = pxlib::PX_create_file(
            pxdoc,
            px_fields_ptr as *mut pxlib::pxfield_t,
            fields.len() as i32,
            c_path.as_ptr(),
            0,
        );

        // We MUST close the document to ensure the header and data are flushed.
        pxlib::PX_close(pxdoc);
        // Delete frees pxdoc and its internal pointers (like the fields array we allocated with malloc).
        pxlib::PX_delete(pxdoc);

        if res >= 0 {
            json!({
                "content": [{ "type": "text", "text": format!("Successfully created table '{}' with {} fields.", table_name, fields.len()) }]
            })
        } else {
            json!({
                "isError": true,
                "content": [{ "type": "text", "text": format!("Failed to create table '{}'.", table_name) }]
            })
        }
    }
}

fn handle_write_record(
    table_name: &str,
    location: &str,
    index: Option<i32>,
    record_data: &Map<String, Value>,
) -> Value {
    let mut full_path = Path::new(location).join(table_name);
    if full_path.extension().is_none() {
        full_path.set_extension("db");
    }

    let path_str = full_path.to_string_lossy();

    unsafe {
        let pxdoc = pxlib::PX_new();
        if pxdoc.is_null() {
            return json!({ "isError": true, "content": [{ "type": "text", "text": "Failed to initialize PX library." }] });
        }

        let c_path = match CString::new(path_str.as_ref()) {
            Ok(c) => c,
            Err(_) => {
                pxlib::PX_delete(pxdoc);
                return json!({ "isError": true, "content": [{ "type": "text", "text": "Invalid table path string." }] });
            }
        };

        if pxlib::PX_open_file(pxdoc, c_path.as_ptr()) < 0 {
            pxlib::PX_delete(pxdoc);
            return json!({ "isError": true, "content": [{ "type": "text", "text": format!("Failed to open table '{}' for writing. Ensure it's not locked.", path_str) }] });
        }

        let num_fields = pxlib::PX_get_num_fields(pxdoc);
        let fields_ptr = pxlib::PX_get_fields(pxdoc);
        let fields_slice = std::slice::from_raw_parts(fields_ptr, num_fields as usize);
        let record_size = pxlib::PX_get_recordsize(pxdoc);
        let mut buf = vec![0u8; record_size as usize];

        if let Some(idx) = index {
            if pxlib::PX_get_record(pxdoc, idx, buf.as_mut_ptr()).is_null() {
                pxlib::PX_close(pxdoc);
                pxlib::PX_delete(pxdoc);
                return json!({ "isError": true, "content": [{ "type": "text", "text": format!("Record at index {} not found.", idx) }] });
            }
        }

        let mut offset = 0;
        for f_idx in 0..num_fields {
            let f = &fields_slice[f_idx as usize];
            let field_name = std::ffi::CStr::from_ptr(f.px_fname)
                .to_string_lossy()
                .into_owned();
            let field_type = f.px_ftype;
            let field_len = f.px_flen;

            if let Some(val) = record_data.get(&field_name) {
                // Add the offset to the base buffer pointer
                let field_ptr = buf.as_mut_ptr().add(offset as usize);
                put_field_value(pxdoc, field_ptr, field_type, field_len, val);
            }

            offset += field_len;
        }

        let res = if let Some(idx) = index {
            pxlib::PX_put_recordn(pxdoc, buf.as_mut_ptr() as *mut std::os::raw::c_char, idx)
        } else {
            pxlib::PX_put_record(pxdoc, buf.as_mut_ptr() as *mut std::os::raw::c_char)
        };

        pxlib::PX_close(pxdoc);
        pxlib::PX_delete(pxdoc);

        if res >= 0 {
            json!({
                "content": [{ "type": "text", "text": format!("Successfully {} record in table '{}'.", if index.is_some() { "updated" } else { "inserted" }, table_name) }]
            })
        } else {
            json!({
                "isError": true,
                "content": [{ "type": "text", "text": format!("Failed to write record to table '{}'.", table_name) }]
            })
        }
    }
}

unsafe fn get_field_value(
    pxdoc: *mut pxlib::pxdoc_t,
    buf_ptr: *mut u8,
    field_type: std::os::raw::c_char,
    field_len: std::os::raw::c_int,
) -> Value {
    match field_type as u32 {
        pxlib::pxfAlpha => {
            let mut val_ptr: *mut std::os::raw::c_char = std::ptr::null_mut();
            if pxlib::PX_get_data_alpha(
                pxdoc,
                buf_ptr as *mut std::os::raw::c_char,
                field_len,
                &mut val_ptr,
            ) >= 0
                && !val_ptr.is_null()
            {
                let s = std::ffi::CStr::from_ptr(val_ptr)
                    .to_string_lossy()
                    .into_owned();
                Value::String(s)
            } else {
                Value::Null
            }
        }
        pxlib::pxfShort => {
            let mut v: std::os::raw::c_short = 0;
            if pxlib::PX_get_data_short(
                pxdoc,
                buf_ptr as *mut std::os::raw::c_char,
                field_len,
                &mut v,
            ) >= 0
            {
                json!(v)
            } else {
                Value::Null
            }
        }
        pxlib::pxfLong | pxlib::pxfAutoInc => {
            let mut v: std::os::raw::c_long = 0;
            if pxlib::PX_get_data_long(
                pxdoc,
                buf_ptr as *mut std::os::raw::c_char,
                field_len,
                &mut v,
            ) >= 0
            {
                json!(v)
            } else {
                Value::Null
            }
        }
        pxlib::pxfNumber | pxlib::pxfCurrency => {
            let mut v: f64 = 0.0;
            if pxlib::PX_get_data_double(
                pxdoc,
                buf_ptr as *mut std::os::raw::c_char,
                field_len,
                &mut v,
            ) >= 0
            {
                json!(v)
            } else {
                Value::Null
            }
        }
        pxlib::pxfLogical => {
            let mut v: std::os::raw::c_char = 0;
            if pxlib::PX_get_data_byte(
                pxdoc,
                buf_ptr as *mut std::os::raw::c_char,
                field_len,
                &mut v,
            ) >= 0
            {
                Value::Bool(v != 0)
            } else {
                Value::Null
            }
        }
        _ => Value::String(format!("<type {}>", field_type)),
    }
}

unsafe fn put_field_value(
    pxdoc: *mut pxlib::pxdoc_t,
    buf_ptr: *mut u8,
    field_type: std::os::raw::c_char,
    field_len: std::os::raw::c_int,
    val: &Value,
) {
    match field_type as u32 {
        pxlib::pxfAlpha => {
            if let Some(s) = val.as_str() {
                if let Ok(c_str) = CString::new(s) {
                    pxlib::PX_put_data_alpha(
                        pxdoc,
                        buf_ptr as *mut std::os::raw::c_char,
                        field_len,
                        c_str.as_ptr() as *mut std::os::raw::c_char,
                    );
                }
            }
        }
        pxlib::pxfShort => {
            if let Some(v) = val.as_i64() {
                pxlib::PX_put_data_short(
                    pxdoc,
                    buf_ptr as *mut std::os::raw::c_char,
                    field_len,
                    v as std::os::raw::c_short,
                );
            }
        }
        pxlib::pxfLong | pxlib::pxfAutoInc => {
            if let Some(v) = val.as_i64() {
                pxlib::PX_put_data_long(
                    pxdoc,
                    buf_ptr as *mut std::os::raw::c_char,
                    field_len,
                    v as std::os::raw::c_int,
                );
            }
        }
        pxlib::pxfNumber | pxlib::pxfCurrency => {
            if let Some(v) = val.as_f64() {
                pxlib::PX_put_data_double(
                    pxdoc,
                    buf_ptr as *mut std::os::raw::c_char,
                    field_len,
                    v,
                );
            }
        }
        pxlib::pxfLogical => {
            if let Some(v) = val.as_bool() {
                pxlib::PX_put_data_byte(
                    pxdoc,
                    buf_ptr as *mut std::os::raw::c_char,
                    field_len,
                    if v { 1 } else { 0 },
                );
            }
        }
        _ => {}
    }
}

fn compare_values(actual: &Value, query: &Value) -> bool {
    match (actual, query) {
        (Value::String(a), Value::String(q)) => a.to_lowercase().contains(&q.to_lowercase()),
        (Value::Number(a), Value::Number(q)) => a == q,
        (Value::Bool(a), Value::Bool(q)) => a == q,
        (Value::String(a), Value::Number(q)) => a == &q.to_string(),
        _ => actual == query,
    }
}
