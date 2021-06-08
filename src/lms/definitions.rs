use crate::lm_ots::definitions::IType;
use crate::lm_ots::definitions::LmotsAlgorithmType;
use crate::lm_ots::definitions::LmotsPrivateKey;
use crate::util::hash::Hasher;
use crate::util::hash::Sha256Hasher;
use crate::util::helper::insert;
use crate::util::helper::read_from_file;
use crate::util::ustr::str32u;
use crate::util::ustr::u32str;
use std::convert::TryInto;
use std::io::Read;
use std::io::Write;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LmsAlgorithmType {
    LmsReserved = 0,
    LmsSha256M32H5 = 5,
    LmsSha256M32H10 = 6,
    LmsSha256M32H15 = 7,
    LmsSha256M32H20 = 8,
    LmsSha256M32H25 = 9,
}

impl LmsAlgorithmType {
    pub fn get_parameter(self) -> LmsAlgorithmParameter {
        LmsAlgorithmParameter::get(self)
    }

    pub fn from_u32(x: u32) -> Option<LmsAlgorithmType> {
        match x {
            0 => Some(LmsAlgorithmType::LmsReserved),
            5 => Some(LmsAlgorithmType::LmsSha256M32H5),
            6 => Some(LmsAlgorithmType::LmsSha256M32H10),
            7 => Some(LmsAlgorithmType::LmsSha256M32H15),
            8 => Some(LmsAlgorithmType::LmsSha256M32H20),
            9 => Some(LmsAlgorithmType::LmsSha256M32H25),
            _ => None,
        }
    }
}

pub struct LmsAlgorithmParameter {
    pub h: u8,
    pub m: u8,
    pub _type: LmsAlgorithmType,
}

impl LmsAlgorithmParameter {
    pub fn get(_type: LmsAlgorithmType) -> Self {
        match _type {
            LmsAlgorithmType::LmsReserved => panic!("Reserved parameter."),
            LmsAlgorithmType::LmsSha256M32H5 => {
                LmsAlgorithmParameter::internal_get(5, 32, LmsAlgorithmType::LmsSha256M32H5)
            }
            LmsAlgorithmType::LmsSha256M32H10 => {
                LmsAlgorithmParameter::internal_get(10, 32, LmsAlgorithmType::LmsSha256M32H10)
            }
            LmsAlgorithmType::LmsSha256M32H15 => {
                LmsAlgorithmParameter::internal_get(15, 32, LmsAlgorithmType::LmsSha256M32H15)
            }
            LmsAlgorithmType::LmsSha256M32H20 => {
                LmsAlgorithmParameter::internal_get(20, 32, LmsAlgorithmType::LmsSha256M32H20)
            }
            LmsAlgorithmType::LmsSha256M32H25 => {
                LmsAlgorithmParameter::internal_get(25, 32, LmsAlgorithmType::LmsSha256M32H25)
            }
        }
    }

    fn internal_get(h: u8, m: u8, _type: LmsAlgorithmType) -> Self {
        LmsAlgorithmParameter { h, m, _type }
    }

    pub fn get_hasher(&self) -> Box<dyn Hasher> {
        match self._type {
            LmsAlgorithmType::LmsReserved => panic!("Reserved parameter."),
            LmsAlgorithmType::LmsSha256M32H5 => Box::new(Sha256Hasher::new()),
            LmsAlgorithmType::LmsSha256M32H10 => Box::new(Sha256Hasher::new()),
            LmsAlgorithmType::LmsSha256M32H15 => Box::new(Sha256Hasher::new()),
            LmsAlgorithmType::LmsSha256M32H20 => Box::new(Sha256Hasher::new()),
            LmsAlgorithmType::LmsSha256M32H25 => Box::new(Sha256Hasher::new()),
        }
    }

    pub fn number_of_lm_ots_keys(&self) -> usize {
        2usize.pow(self.h as u32)
    }
}

#[allow(non_snake_case)]
#[derive(Debug, PartialEq, Eq)]
pub struct LmsPrivateKey {
    pub lms_type: LmsAlgorithmType,
    pub lm_ots_type: LmotsAlgorithmType,
    pub key: Vec<LmotsPrivateKey>,
    pub I: IType,
    pub q: u32,
}

#[allow(non_snake_case)]
impl LmsPrivateKey {
    pub fn new(
        lms_type: LmsAlgorithmType,
        lmots_type: LmotsAlgorithmType,
        key: Vec<LmotsPrivateKey>,
        I: IType,
    ) -> Self {
        LmsPrivateKey {
            lms_type,
            lm_ots_type: lmots_type,
            key,
            I,
            q: 0,
        }
    }

    pub fn use_lmots_private_key(&mut self) -> Result<&LmotsPrivateKey, &'static str> {
        if self.q as usize > self.key.len() {
            return Err("All private keys already used.");
        }
        self.q += 1;
        Ok(&self.key[self.q as usize - 1])
    }

    pub fn to_binary_representation(&self) -> Vec<u8> {
        let mut result = Vec::new();

        insert(&u32str(self.lms_type as u32), &mut result);
        insert(&u32str(self.lm_ots_type as u32), &mut result);
        insert(&self.I, &mut result);
        insert(&u32str(self.q), &mut result);

        let keys = self
            .key
            .iter()
            .map(|key| key.get_flat_key())
            .flatten()
            .collect::<Vec<u8>>();

        insert(&keys, &mut result);

        result
    }

    pub fn to_file(&self, filename: &str) -> Result<(), std::io::Error> {
        let binary_representation = self.to_binary_representation();

        let mut file = std::fs::File::open(filename)?;

        file.write_all(&binary_representation)?;
        Ok(())
    }

    pub fn from_file(filename: &str) -> Self {
        let mut data = std::fs::File::open(filename).expect("Can not open file.");

        let mut buf = [0u8; 4];

        read_from_file(&mut data, &mut buf);
        let lms_type = str32u(&buf);
        let lms_type = LmsAlgorithmType::from_u32(lms_type).expect("Valid Lmots Type");
        let lms_parameter = lms_type.get_parameter();

        read_from_file(&mut data, &mut buf);
        let lm_ots_type = str32u(&buf);
        let lm_ots_type = LmotsAlgorithmType::from_u32(lm_ots_type).expect("Valid LM OTS Type");
        let lm_ots_parameter = lm_ots_type.get_parameter();

        let mut initial_buf = [0u8; 16];
        read_from_file(&mut data, &mut initial_buf);

        read_from_file(&mut data, &mut buf);
        let q = str32u(&buf);

        let mut data_to_end: Vec<u8> = Vec::new();
        data.read_to_end(&mut data_to_end)
            .expect("Could not read file.");

        let mut keys: Vec<LmotsPrivateKey> = Vec::new();

        for _ in 0..lms_parameter.number_of_lm_ots_keys() {
            let mut current_key: Vec<Vec<u8>> = Vec::new();

            // vec![vec![0u8; parameter.n as usize]; parameter.p as usize];

            for _ in 0..lm_ots_parameter.p {
                let mut x = Vec::new();
                for _ in 0..lm_ots_parameter.n {
                    x.push(data_to_end[0]);
                    data_to_end.remove(0);
                }
                current_key.push(x);
            }

            // Append key
            let lmots_private_key =
                LmotsPrivateKey::new(initial_buf, u32str(q), lm_ots_parameter, current_key);
            keys.push(lmots_private_key);
        }

        LmsPrivateKey {
            lms_type,
            lm_ots_type,
            key: keys,
            I: initial_buf,
            q,
        }
    }
}

#[allow(non_snake_case)]
pub struct LmsPublicKey {
    pub lm_ots_type: LmotsAlgorithmType,
    pub lms_type: LmsAlgorithmType,
    pub key: Vec<u8>,
    pub tree: Option<Vec<Vec<u8>>>,
    pub I: IType,
}

#[allow(non_snake_case)]
impl LmsPublicKey {
    pub fn new(
        public_key: Vec<u8>,
        tree: Vec<Vec<u8>>,
        lm_ots_type: LmotsAlgorithmType,
        lms_type: LmsAlgorithmType,
        I: IType,
    ) -> Self {
        LmsPublicKey {
            lm_ots_type,
            lms_type,
            key: public_key,
            tree: Some(tree),
            I,
        }
    }

    pub fn to_binary_representation(&self) -> Vec<u8> {
        let mut result = Vec::new();

        insert(&u32str(self.lms_type as u32), &mut result);
        insert(&u32str(self.lm_ots_type as u32), &mut result);
        insert(&self.I, &mut result);
        insert(&self.key, &mut result);

        result
    }

    pub fn from_binary_representation(data: Vec<u8>) -> Option<Self> {
        // Parsing like desribed in 5.4.2
        if data.len() < 8 {
            return None;
        }

        let mut data_index = 0;

        let pubtype = str32u(data[data_index..data_index + 4].try_into().unwrap());
        data_index += 4;

        let lms_type = match LmsAlgorithmType::from_u32(pubtype) {
            None => return None,
            Some(x) => x,
        };

        let ots_typecode = str32u(data[data_index..data_index + 4].try_into().unwrap());
        data_index += 4;

        let lm_ots_type = match LmotsAlgorithmType::from_u32(ots_typecode) {
            None => return None,
            Some(x) => x,
        };

        let lm_parameter = lms_type.get_parameter();

        if data.len() - data_index == 24 + lm_parameter.m as usize {
            return None;
        }

        let mut initial: IType = [0u8; 16];
        initial.clone_from_slice(&data[data_index..data_index + 16]);
        data_index += 16;

        let mut key: Vec<u8> = Vec::new();

        for i in 0..lm_parameter.m {
            key.push(data[data_index + i as usize]);
        }

        let public_key = LmsPublicKey {
            lms_type,
            lm_ots_type,
            I: initial,
            key,
            tree: None,
        };

        Some(public_key)
    }
}

#[cfg(test)]
mod tests {
    use crate::lms::keygen::generate_private_key;

    use super::LmsPrivateKey;

    #[test]
    fn private_key_serialization_deserialisation() {
        let private_key = generate_private_key(
            crate::LmsAlgorithmType::LmsSha256M32H5,
            crate::LmotsAlgorithmType::LmotsSha256N32W1,
        );

        let temp_filename = "temp.priv";

        private_key.to_file(temp_filename).unwrap();

        let private_key_from_file = LmsPrivateKey::from_file(temp_filename);

        assert!(private_key == private_key_from_file);
    }
}
