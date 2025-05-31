/*
 * Copyright 2019, 2020, 2022, 2025 sukawasatoru
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::{cmp, fmt};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileVersion {
    major: i32,
    minor: i32,
    patch: i32,
}

impl From<[i32; 3]> for FileVersion {
    fn from(value: [i32; 3]) -> Self {
        Self {
            major: value[0],
            minor: value[1],
            patch: value[2],
        }
    }
}

impl std::str::FromStr for FileVersion {
    type Err = anyhow::Error;

    fn from_str(version: &str) -> Result<Self, Self::Err> {
        let v: Vec<&str> = version.split('.').collect::<Vec<_>>();
        Ok(FileVersion::from([
            v[0].parse()?,
            v[1].parse()?,
            v[2].parse()?,
        ]))
    }
}

impl fmt::Display for FileVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl PartialOrd<FileVersion> for FileVersion {
    fn partial_cmp(&self, other: &FileVersion) -> Option<cmp::Ordering> {
        let major = self.major.cmp(&other.major);
        if major != cmp::Ordering::Equal {
            return Some(major);
        }

        let minor = self.minor.cmp(&other.minor);
        if minor != cmp::Ordering::Equal {
            return Some(minor);
        }

        let patch = self.patch.cmp(&other.patch);
        if patch != cmp::Ordering::Equal {
            return Some(patch);
        }

        Some(cmp::Ordering::Equal)
    }
}

impl serde::Serialize for FileVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{}.{}.{}", self.major, self.minor, self.patch))
    }
}

impl<'de> serde::Deserialize<'de> for FileVersion {
    fn deserialize<D>(deserializer: D) -> Result<FileVersion, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct StrVisitor;
        impl<'de> serde::de::Visitor<'de> for StrVisitor {
            type Value = FileVersion;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a string")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                v.parse::<FileVersion>().map_err(serde::de::Error::custom)
            }
        }

        deserializer.deserialize_str(StrVisitor)
    }
}
