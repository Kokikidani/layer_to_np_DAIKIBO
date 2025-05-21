use std::{ fs::File, io::{ Error, Write } };

/// Writes the contents of a 2D vector to a file.
/// 
/// # Arguments
///
/// * `file_name` - A reference to a string representing the name of the file.
/// * `data` - A reference to a 2D vector containing data to be written to the file.
///
/// # Returns
///
/// * `Result<(), Error>` - Returns `Ok(())` if successful, or an `Error` if an error occurs.
///
pub fn output_file_from_2dvec<T: std::fmt::Display>(
    file_name: &String,
    data: &[Vec<T>]
) -> Result<(), Error> {
    let mut output = File::create(file_name)?;

    for d in data {
        for v in d {
            output.write_all(format!("{} ", v).as_bytes())?;
        }
        output.write_all("\n".as_bytes())?;
    }

    Ok(())
}

/// Creates a file and handles any errors that occur during file creation.
///
/// # Arguments
///
/// * `filename` - A reference to a string representing the name of the file.
///
/// # Returns
///
/// * `File` - Returns a `File` object if successful. If file creation fails, it prints an error message and panics.
///
pub fn get_file(filename: &String) -> File {
    File::create(filename).unwrap_or_else(|err|{
        eprintln!("Failed to open file: {}\nfilepath: {}", err, filename);
        panic!();
    })
}