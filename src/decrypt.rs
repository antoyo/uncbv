/*
 * Copyright (C) 2016  Boucher, Antoni <bouanto@zoho.com>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

//! CBV decryption functions.

use std::io::{self, BufReader, Read, Write};

use des;

const BUFFER_SIZE: usize = 4096;
const PASSWORD_LEN: usize = 8;

/// Create the decryption key from the password.
fn create_key(password: &str) -> [u8; PASSWORD_LEN] {
    if password.len() < PASSWORD_LEN {
        // If the password length is lesser than 8, the password is repeated until its len is 8.
        let mut new_password = password.to_string();
        while new_password.len() < PASSWORD_LEN {
            new_password.push_str(password);
        }
        copy_into_array(new_password.as_bytes())
    }
    else if password.len() > PASSWORD_LEN {
        // If the password length is greater than 8, the password is hashed.
        let mut key = [0; PASSWORD_LEN];
        let bytes = password.as_bytes();

        for (i, &byte) in bytes.iter().enumerate() {
            let index = i % 8;
            key[index] *= 2;
            key[index] ^= byte;
        }

        key
    }
    else {
        copy_into_array(password.as_bytes())
    }
}

/// Copy a slice into an array.
fn copy_into_array(slice: &[u8]) -> [u8; PASSWORD_LEN] {
    let mut array = [0; PASSWORD_LEN];
    for (index, &byte) in slice.iter().enumerate() {
        array[index] = byte;
    }
    array
}

/// Decrypt the file into `output`.
pub fn decrypt<R: Read>(reader: R, password: &str, output: &mut Write) -> Result<(), io::Error> {
    let key = create_key(password);

    let mut reader = BufReader::new(reader);
    let mut buffer = [0; BUFFER_SIZE];

    while let Ok(byte_count) = reader.read(&mut buffer) {
        if byte_count == 0 {
            break;
        }

        let result = des::decrypt(&buffer, &key);

        try!(output.write_all(&result[..byte_count]));
    }

    Ok(())
}
