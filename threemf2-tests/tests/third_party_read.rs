#[cfg(test)]
mod tests {
    mod test_utilities;

    use std::fs::File;
    use std::path::PathBuf;

    #[test]
    pub fn can_load_thirdparty_3mf_package() {
        let fixtures = test_utilities::get_test_fixtures();

        for fixture in fixtures {
            use threemf2::package::ThreemfPackage;

            if fixture.skip_test || fixture.large_test {
                continue;
            }

            let mut folder_path = PathBuf::from("./tests/data/third-party/");
            if fixture.large_test {
                folder_path = folder_path.join("lfs");
            }
            let filepath = folder_path.join(fixture.filepath.clone());
            // println!("{:?}", filepath);
            let file = File::open(&filepath).unwrap();

            let package =
                ThreemfPackage::from_reader_with_memory_optimized_deserializer(file, true);

            match package {
                Ok(threemf) => {
                    assert!(!threemf.content_types.defaults.is_empty());
                    assert!(!threemf.relationships.is_empty());
                    assert!(!threemf.root.build.item.is_empty());
                }
                Err(err) => {
                    panic!(
                        "Failed to read the file: {:?} with err: {:?}",
                        &filepath, err
                    );
                }
            }
        }
    }

    #[test]
    pub fn unpack_thirdparty_3mf_package() {
        use threemf2::package::CachePolicy;
        use threemf2::package::ThreemfPackageLazyReader;

        let fixtures = test_utilities::get_test_fixtures();

        for fixture in fixtures {
            if fixture.skip_test {
                continue;
            }

            let mut folder_path = PathBuf::from("./tests/data/third-party/");
            if fixture.large_test {
                folder_path = folder_path.join("lfs");
            }

            let filepath = folder_path.join(fixture.filepath);
            let file = File::open(&filepath).unwrap();

            let package = ThreemfPackageLazyReader::from_reader_with_memory_optimized_deserializer(
                file,
                CachePolicy::NoCache,
            );

            match package {
                Ok(threemf) => {
                    assert!(!threemf.content_types().defaults.is_empty());
                    assert!(!threemf.relationships().is_empty());
                    assert_eq!(threemf.root_model_path().as_str(), "/3D/3dmodel.model");
                }
                Err(err) => {
                    panic!(
                        "Failed to read the file: {:?} with err: {:?}",
                        &filepath, err
                    );
                }
            }
        }
    }
}
