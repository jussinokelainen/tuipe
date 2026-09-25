use color_eyre::Result;

/*
* A struct to pass valid flags to the parser.
*
* flags: normal flags, basically like booleans.
* valued_flags: flags that take a value after them, for example '--depth 1'
* optional_flags: flags that try to take a value with them, but also work without
*
* Optional flags will take the argument after them as it's value unless they are
* the last argument, or the argument after them is another flag (starts with '-')
*/
pub struct Flagset {
    pub flags: Vec<&'static str>,
    pub value_flags: Vec<&'static str>,
    pub opt_flags: Vec<&'static str>,
}

/*
* A struct that gets returned by parse_args. Only flags that were contained in
* the arguments are in the returned struct.
*
* flags: normal flags and optional flags that didn't get a value with them
* valued_flags: tuple that contains the flag, and it's value
* normal_str: strings that were in the arguments, strings that were the values
*             of optional or value flags are not contained here.
*/
pub struct Arguments {
    pub flags: Vec<String>,
    pub valued_flags: Vec<(String, String)>,
    pub normal_str: Vec<String>,
}

/*
* A function to parse arguments, returns them as a struct with separate vectors
* for normal flags, flags that take values and normal strings.
*
* As a parameter requires the arguments that will be parsed, and a Flagset struct
* that contains valid arguments. If all flags are to be accepted, all structs in
* the given Flagset struct must be empty. If that is the case, all flags will
* be parsed and returned as normal boolean-style flags
*
* Returns an Arguments struct and a bool, which is true if there was an invalid
* flag, e.g. a flag was called that isn't in the vector of valid flags, or a
* flag that requires a value was the last argument
*/
pub fn parse_args(input: Flagset) -> Result<Arguments> {
    // Get all arguments and make them into a vector of Strings
    let env_args = std::env::args();
    let mut input_args = vec![];
    for arg in env_args.enumerate() {
        input_args.push(arg.1)
    }

    // Remove the program name from arguments
    input_args.remove(0);

    let mut has_invalid = false;
    let mut accept_all = false;
    let mut flags = Arguments {
        flags: vec![],
        valued_flags: vec![],
        normal_str: vec![],
    };

    // If no specific flags are given by the caller, all flags will be valid
    if input.flags.is_empty() && input.value_flags.is_empty() && input.opt_flags.is_empty() {
        accept_all = true;
    }

    // Check if a flag takes a value as a parameter
    let requires_value = |flag: String, num: usize| -> bool {
        if input.value_flags.contains(&flag.as_str()) {
            return true;
        }

        if input.opt_flags.contains(&flag.as_str())
            && (num + 1) < input_args.len()
            && !input_args[num + 1].starts_with('-')
        {
            return true;
        }

        false
    };

    // Handles a single flag
    let mut handle_flag = |flag: String, num: &mut usize| {
        if requires_value(flag.clone(), *num) {
            // Flag requires a value, check that it is not
            // the last argument before taking the value
            if (*num + 1) < input_args.len() {
                // Get the next argument to be the value for this flag
                let new_flag = (flag.clone().to_string(), input_args[*num + 1].clone());
                flags.valued_flags.push(new_flag);
                *num += 1;
            } else {
                has_invalid = true
            }
        } else {
            if input.flags.contains(&flag.as_str())
                || input.opt_flags.contains(&flag.as_str())
                || accept_all
            {
                flags.flags.push(flag.clone().to_string());
            } else {
                has_invalid = true;
            }
        }
    };

    let mut count = 0;
    while count < input_args.len() {
        if input_args[count].starts_with("--") {
            let flag = &input_args[count][2..];
            handle_flag(flag.to_string(), &mut count)
        } else if input_args[count].starts_with('-') {
            let flag = &input_args[count][1..];
            for c in flag.chars() {
                handle_flag(c.to_string(), &mut count)
            }
        } else {
            flags.normal_str.push(input_args[count].clone());
        }
        count += 1;
    }

    if has_invalid {
        Err(color_eyre::eyre::eyre!("Invalid Flags."))
    } else {
        Ok(flags)
    }
}
