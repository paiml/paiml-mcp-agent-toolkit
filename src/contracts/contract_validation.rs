/// Trait for contract validation
pub trait ContractValidation {
    fn validate(&self) -> Result<(), ContractError>;
}

impl ContractValidation for BaseAnalysisContract {
    fn validate(&self) -> Result<(), ContractError> {
        PathValidator::ensure_exists(&self.path)
            .map_err(|_| ContractError::PathNotFound(self.path.clone()))?;

        if self.timeout == 0 {
            return Err(ContractError::InvalidTimeout);
        }

        if let Some(top_files) = self.top_files {
            if top_files > 1000 {
                return Err(ContractError::TooManyFiles(top_files));
            }
        }

        Ok(())
    }
}

impl ContractValidation for AnalyzeComplexityContract {
    fn validate(&self) -> Result<(), ContractError> {
        self.base.validate()?;

        if let Some(max_halstead) = self.max_halstead {
            if max_halstead <= 0.0 {
                return Err(ContractError::InvalidValue(
                    "max_halstead must be positive".into(),
                ));
            }
        }

        Ok(())
    }
}

impl ContractValidation for AnalyzeSatdContract {
    fn validate(&self) -> Result<(), ContractError> {
        self.base.validate()
    }
}

impl ContractValidation for AnalyzeDeadCodeContract {
    fn validate(&self) -> Result<(), ContractError> {
        self.base.validate()?;

        if self.max_percentage < 0.0 || self.max_percentage > 100.0 {
            return Err(ContractError::InvalidValue(
                "max_percentage must be 0-100".into(),
            ));
        }

        Ok(())
    }
}

impl ContractValidation for AnalyzeTdgContract {
    fn validate(&self) -> Result<(), ContractError> {
        self.base.validate()?;

        if self.threshold < 0.0 {
            return Err(ContractError::InvalidValue(
                "threshold must be non-negative".into(),
            ));
        }

        Ok(())
    }
}

impl ContractValidation for AnalyzeLintHotspotContract {
    fn validate(&self) -> Result<(), ContractError> {
        self.base.validate()?;

        if self.max_density < 0.0 {
            return Err(ContractError::InvalidValue(
                "max_density must be non-negative".into(),
            ));
        }

        if self.min_confidence < 0.0 || self.min_confidence > 1.0 {
            return Err(ContractError::InvalidValue(
                "min_confidence must be 0-1".into(),
            ));
        }

        Ok(())
    }
}

impl ContractValidation for AnalyzeEntropyContract {
    fn validate(&self) -> Result<(), ContractError> {
        self.base.validate()?;

        // Validate severity level if provided
        if let Some(severity) = &self.min_severity {
            match severity.as_str() {
                "low" | "medium" | "high" => {}
                _ => {
                    return Err(ContractError::InvalidValue(
                        "min_severity must be 'low', 'medium', or 'high'".into(),
                    ))
                }
            }
        }

        // Validate top_violations if provided
        if let Some(violations) = self.top_violations {
            if violations > 1000 {
                return Err(ContractError::TooManyFiles(violations));
            }
        }

        // Validate file path if provided
        if let Some(file) = &self.file {
            PathValidator::ensure_exists(file)
                .map_err(|_| ContractError::PathNotFound(file.clone()))?;
        }

        Ok(())
    }
}

impl ContractValidation for QualityGateContract {
    fn validate(&self) -> Result<(), ContractError> {
        self.base.validate()?;

        if let Some(file) = &self.file {
            PathValidator::ensure_exists(file)
                .map_err(|_| ContractError::PathNotFound(file.clone()))?;
        }

        Ok(())
    }
}

impl ContractValidation for RefactorAutoContract {
    fn validate(&self) -> Result<(), ContractError> {
        PathValidator::ensure_file(&self.file)
            .map_err(|_| ContractError::PathNotFound(self.file.clone()))?;

        if self.timeout == 0 {
            return Err(ContractError::InvalidTimeout);
        }

        if self.target_complexity == 0 {
            return Err(ContractError::InvalidValue(
                "target_complexity must be > 0".into(),
            ));
        }

        Ok(())
    }
}
