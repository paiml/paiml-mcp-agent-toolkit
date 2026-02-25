#[test]
fn test_backend_auto_select() {
    let backend = BackendSelector::auto_select();

    #[cfg(feature = "analytics-simd")]
    assert_eq!(backend, Backend::Simd);

    #[cfg(not(feature = "analytics-simd"))]
    assert_eq!(backend, Backend::Scalar);
}

#[test]
fn test_generate_dataset() {
    let dataset = generate_test_dataset(1000);
    assert_eq!(dataset.len(), 1000);

    // Check for mix of positive/negative values
    let positive = dataset.iter().filter(|&&x| x > 0.0).count();
    let negative = dataset.iter().filter(|&&x| x < 0.0).count();
    assert!(positive > 0);
    assert!(negative > 0);
}

#[test]
fn test_mean_and_std() {
    let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let (mean, std) = mean_and_std(&values);

    // Mean should be 3.0
    assert!((mean - 3.0).abs() < 0.01);

    // Std should be ~1.58 (sample std)
    assert!((std - 1.58).abs() < 0.1);
}

#[test]
fn test_mean_and_std_empty() {
    let values: Vec<f64> = vec![];
    let (mean, std) = mean_and_std(&values);
    assert_eq!(mean, 0.0);
    assert_eq!(std, 0.0);
}

#[test]
fn test_compute_avg_scalar() {
    let dataset = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let avg = compute_avg(&dataset, Backend::Scalar).unwrap();
    assert!((avg - 3.0).abs() < 0.01);
}

#[test]
#[cfg(feature = "analytics-simd")]
fn test_compute_avg_simd() {
    let dataset = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let avg = compute_avg(&dataset, Backend::Simd).unwrap();
    assert!((avg - 3.0).abs() < 0.01);
}
