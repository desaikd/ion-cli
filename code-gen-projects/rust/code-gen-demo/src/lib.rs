pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;
    use ion_rs::Element;
    // use ion_rs::IonType;
    use ion_rs::lazy::encoder::text::v1_0::writer::LazyRawTextWriter_1_0;
    use ion_rs::lazy::encoder::value_writer::internal::MakeValueWriter;
    use ion_rs::lazy::encoder::LazyRawWriter;
    use ion_rs::lazy::reader::LazyReader;
    use ion_rs::lazy::streaming_raw_reader::IonInput;
    use std::fs;
    include!(concat!(env!("OUT_DIR"), "/ion_generated_code.rs"));

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn test_roundtrip_generated_code_structs_with_fields() -> IonResult<()> {
        let ion_string = fs::read_to_string(&format!(
            "{}/../../input/struct_with_fields.ion",
            env!("CARGO_MANIFEST_DIR")
        ))?;
        let mut reader = LazyReader::new(ion_string.clone());
        // read given Ion value using Ion reader
        let value = reader.expect_next()?.read()?;
        let structs_with_fields: StructWithFields = StructWithFields::read_from(value)?;

        // write the generated abstract data type using Ion writer
        let mut buffer = Vec::new();
        let mut writer = LazyRawTextWriter_1_0::new(&mut buffer)?;
        let value_writer = writer.make_value_writer();
        structs_with_fields.write_as_ion(value_writer)?;
        writer.flush()?;
        // compare given Ion value with round tripped Ion value written using abstract data type's `write_to` API
        assert_eq!(
            Element::read_one(writer.output().as_slice())?,
            (Element::read_one(&ion_string)?)
        );

        Ok(())
    }

    #[test]
    fn test_roundtrip_generated_code_nested_structs() -> IonResult<()> {
        let ion_string = fs::read_to_string(&format!(
            "{}/../../input/nested_struct.ion",
            env!("CARGO_MANIFEST_DIR")
        ))?;
        let mut reader = LazyReader::new(ion_string.clone());
        // read given Ion value using Ion reader
        let value = reader.expect_next()?.read()?;
        let nested_struct: NestedStruct = NestedStruct::read_from(value)?;

        // write the generated abstract data type using Ion writer
        let mut buffer = Vec::new();
        let mut writer = LazyRawTextWriter_1_0::new(&mut buffer)?;
        let value_writer = writer.make_value_writer();
        nested_struct.write_as_ion(value_writer)?;
        writer.flush()?;
        // compare given Ion value with round tripped Ion value written using abstract data type's `write_to` API
        assert_eq!(
            Element::read_one(writer.output().as_slice())?,
            (Element::read_one(&ion_string)?)
        );

        Ok(())
    }
}
