// Test fixture with exact duplicates

fn authenticate_user(username: &str, password: &str) -> Result<User, AuthError> {
    let user = fetch_user(username)?;
    if !verify_password(password, &user.password_hash) {
        return Err(AuthError::InvalidCredentials);
    }
    Ok(user)
}

fn validate_token(token: &str) -> Result<Claims, AuthError> {
    let claims = decode_jwt(token)?;
    if claims.exp < current_time() {
        return Err(AuthError::TokenExpired);
    }
    Ok(claims)
}

// Exact duplicate of authenticate_user
fn authenticate_user(username: &str, password: &str) -> Result<User, AuthError> {
    let user = fetch_user(username)?;
    if !verify_password(password, &user.password_hash) {
        return Err(AuthError::InvalidCredentials);
    }
    Ok(user)
}

// Another validation function with similar pattern
fn validate_session(session_id: &str) -> Result<Session, AuthError> {
    let session = fetch_session(session_id)?;
    if session.expired_at < current_time() {
        return Err(AuthError::SessionExpired);
    }
    Ok(session)
}