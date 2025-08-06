use image::codecs::png::PngEncoder;
use image::{ColorType, ImageEncoder, Luma};
use otpauth::TOTP;
use qrcode::QrCode;
use rand::TryRngCore;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::io::Cursor;
use thiserror::Error;
use tokio_postgres::types::private::BytesMut;
use tokio_postgres::types::{FromSql, IsNull, ToSql, Type};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema, Copy, Clone, Eq, PartialEq)]
#[repr(i32)]
pub enum TfaStatus {
    Pending = 1,
    Enabled = 2,
}

impl ToSql for TfaStatus {
    fn to_sql(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        (*self as i16).to_sql(ty, out)
    }

    fn accepts(ty: &Type) -> bool {
        <i16 as ToSql>::accepts(ty)
    }

    tokio_postgres::types::to_sql_checked!();
}

impl<'a> FromSql<'a> for TfaStatus {
    fn from_sql(ty: &Type, raw: &'a [u8]) -> Result<Self, Box<dyn Error + Sync + Send>> {
        let value = i16::from_sql(ty, raw)?;
        match value {
            1 => Ok(TfaStatus::Pending),
            2 => Ok(TfaStatus::Enabled),
            _ => Err("Valor de status desconocido".into()),
        }
    }

    fn accepts(ty: &Type) -> bool {
        <i16 as FromSql>::accepts(ty)
    }
}

#[derive(Debug, Error)]
pub enum TfaError {
    #[error("Error generando el código TOTP")]
    GenerateTotpError,
    #[error("El código TOTP no es válido")]
    InvalidTotpCode,
    #[error("Error generando el QR code: {0}")]
    QrCodeError(String),
    #[error("Error escribiendo imagen PNG: {0}")]
    ImageWriteError(String),
}

#[derive(Debug)]
pub struct Tfa {
    secret: String,
    totp: TOTP,
}

impl Tfa {
    pub fn generate_secret() -> Result<String, Box<dyn std::error::Error>> {
        use rand::rngs::OsRng;
        const SECRET_LEN: usize = 10;

        let mut bytes = [0u8; SECRET_LEN];
        OsRng.try_fill_bytes(&mut bytes)?;

        let secret = base32::encode(base32::Alphabet::RFC4648 { padding: false }, &bytes);
        Ok(secret)
    }

    pub fn new(secret: &str) -> Self {
        let totp = TOTP::new(secret);
        Tfa {
            secret: secret.to_string(),
            totp,
        }
    }

    pub fn generate_totp(&self, timestamp: u64) -> Result<u32, TfaError> {
        Ok(self.totp.generate(30, timestamp))
    }

    pub fn verify_totp(&self, code: u32, timestamp: u64) -> Result<bool, TfaError> {
        Ok(self.totp.verify(code, 30, timestamp))
    }

    pub fn generate_qr_code(&self, account: &str, issuer: &str) -> Result<Vec<u8>, TfaError> {
        let url = self.build_otpauth_url(account, issuer);

        let code = QrCode::new(url).map_err(|e| TfaError::QrCodeError(e.to_string()))?;
        let image = code.render::<Luma<u8>>().build();

        let mut buf = Cursor::new(Vec::new());
        let encoder = PngEncoder::new(&mut buf);
        encoder
            .write_image(&image, image.width(), image.height(), ColorType::L8.into())
            .map_err(|e| TfaError::ImageWriteError(e.to_string()))?;

        Ok(buf.into_inner())
    }

    fn build_otpauth_url(&self, account: &str, issuer: &str) -> String {
        format!(
            "otpauth://totp/{}:{}?secret={}&issuer={}&period=30",
            urlencoding::encode(issuer),
            urlencoding::encode(account),
            self.secret,
            urlencoding::encode(issuer)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Error obteniendo timestamp")
            .as_secs()
    }

    #[test]
    fn test_generate_and_verify_totp() {
        let secret = "JBSWY3DPEHPK3PXP";
        let tfa = Tfa::new(secret);

        let timestamp = current_timestamp();
        let code = tfa.generate_totp(timestamp).expect("Error generando TOTP");

        let is_valid = tfa
            .verify_totp(code, timestamp)
            .expect("Error verificando TOTP");
        assert!(is_valid, "El código TOTP debería ser válido");
    }

    #[test]
    fn test_invalid_totp() {
        let secret = "JBSWY3DPEHPK3PXP";
        let tfa = Tfa::new(secret);

        let timestamp = current_timestamp();
        let is_valid = tfa
            .verify_totp(123456, timestamp)
            .expect("Error verificando TOTP");

        assert!(!is_valid, "Un código TOTP incorrecto debería fallar");
    }

    #[test]
    fn test_generate_qr_code() {
        let secret = "JBSWY3DPEHPK3PXP";
        let tfa = Tfa::new(secret);

        let qr_bytes = tfa
            .generate_qr_code("user@example.com", "TestService")
            .expect("Error generando QR Code");

        assert!(!qr_bytes.is_empty(), "El QR Code no debería estar vacío");
    }
}
