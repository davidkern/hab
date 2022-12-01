use nom::{
    bytes::streaming::{take, tag},
    IResult
};

/// Takes a u8 from the byte slice
fn take_u8(input: &[u8]) -> IResult<&[u8], u8> {
    take(1usize)(input).map(|(input, output)| (input, output[0]))
}

// MK3 frame
// <Length> 0xff <Command> <Data_0> ... <Data_n-1> <Checksum>
fn mk3_frame(input: &[u8]) -> IResult<&[u8], &[u8]> {
    let (input, length) = take_u8(input)?;
    let (input, _) = tag(&[0xff])(input)?;
    let (input, data) = take(length as usize)(input)?;
    let (input, checksum) = take_u8(input)?;

    // if length has MSB set, then led status is appended
    
    Ok((input, input))
}
