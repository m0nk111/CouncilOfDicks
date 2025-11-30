# Cryptographic Signatures for AI Responses

Council Of Dicks implements **Ed25519 digital signatures** to ensure every AI response is:
- **Authentic** - provably came from a specific peer
- **Immutable** - cannot be tampered with
- **Non-repudiable** - peer cannot deny they sent it

## Why Signatures?

In a decentralized P2P network, any peer can claim to be anyone. Cryptographic signatures solve this:

```
❌ Without signatures:
Malicious Peer: "I'm GPT-4 and the answer is X"
→ No way to verify identity
→ Responses can be tampered with
→ No accountability

✅ With signatures:
Peer: "Here's my response: X"
Signed with: Ed25519 signature + public key
→ Mathematically proven to come from this peer
→ Any tampering breaks the signature
→ Permanent cryptographic proof
```

## How It Works

### 1. Key Generation

Each Council node generates a unique Ed25519 keypair on first run:

```rust
// Generate new identity
let identity = SigningIdentity::generate();

// Save to disk
identity.save(PathBuf::from("./council_identity.key"))?;
```

**Keypair Storage:**
- Private key: `council_identity.key` (32 bytes, keep secret!)
- Public key: Derived from private key, shared with everyone

**Security:**
- Ed25519 provides 128-bit security (equivalent to 3072-bit RSA)
- Private key never leaves the machine
- Faster than RSA/PGP while more secure

### 2. Signing Responses

When an AI generates a response, it's immediately signed:

```rust
let response = "The answer is 42";
let signed = identity.sign(response);

// SignedMessage contains:
// - content: "The answer is 42"
// - signature: Base64 encoded Ed25519 signature
// - public_key: Base64 encoded public key
// - timestamp: Unix timestamp (prevents replay attacks)
```

**Canonical Format:**
```
message = "{content}|{timestamp}"
signature = Ed25519_sign(private_key, message)
```

Including timestamp prevents **replay attacks** (reusing old valid signatures).

### 3. Verification

Anyone can verify a signed response:

```rust
let verified = verify_signed_message(&signed)?;

if verified {
    println!("✅ Signature valid - response is authentic");
} else {
    println!("❌ Signature invalid - response tampered or fake");
}
```

**Verification Process:**
1. Decode public key from Base64
2. Decode signature from Base64
3. Reconstruct canonical message: `"{content}|{timestamp}"`
4. Verify signature using Ed25519 algorithm
5. Return `true` if valid, `false` if tampered

## API Usage

### Backend (Rust)

```rust
use crate::crypto::{SigningIdentity, verify_signed_message, SignedMessage};

// Generate identity
let identity = SigningIdentity::generate();

// Sign message
let signed = identity.sign("AI response here");

// Verify
let valid = verify_signed_message(&signed)?;

// Get fingerprint (for display)
let fingerprint = public_key_fingerprint(&identity.public_key_base64())?;
// Returns: "A1B2C3D4E5F6G7H8" (16 chars)
```

### Frontend (TypeScript)

```typescript
import { getPublicKey, verifySignature, getKeyFingerprint } from './api';

// Get our public key
const pubkey = await getPublicKey();

// Verify a response
const valid = await verifySignature(
  content,
  signature,
  publicKey,
  timestamp
);

// Get fingerprint for display
const fingerprint = await getKeyFingerprint();
```

## Protocol Integration

### CouncilResponse Structure

```rust
pub struct CouncilResponse {
    pub model_name: String,
    pub response: String,
    pub peer_id: String,
    pub timestamp: u64,
    pub signature: Option<String>,  // Base64 Ed25519 signature
    pub public_key: Option<String>, // Base64 public key
}
```

**Why Optional?**
- Legacy responses (before signatures were implemented)
- Testing/development without crypto overhead
- Backwards compatibility

**Production:** Always require signatures!

### P2P Message Protocol

```rust
Response {
    question_id: String,
    model_name: String,
    signed_response: SignedMessage, // Complete signed package
    peer_id: String,
}
```

## Security Properties

### Cryptographic Guarantees

1. **Authenticity**
   - Only holder of private key can create valid signatures
   - Public key uniquely identifies the signer
   - Mathematically proven link between message and signer

2. **Integrity**
   - Any change to content breaks the signature
   - Timestamp prevents replay attacks
   - Hash-based verification is deterministic

3. **Non-repudiation**
   - Signer cannot deny creating the signature
   - Signature is permanent cryptographic proof
   - Third parties can verify independently

### Attack Resistance

**❌ Cannot do:**
- Forge signatures without private key
- Modify signed content without detection
- Replay old signatures (timestamp check)
- Impersonate another peer
- Perform man-in-the-middle attacks (if P2P uses TLS)

**⚠️ Can do (out of scope):**
- Read signed messages (they're not encrypted!)
- DDoS the network (rate limiting needed)
- Sybil attacks (reputation system mitigates)

## Key Management

### Storage

```bash
# Keypair location
./council_identity.key  # 32 bytes, Ed25519 private key
```

**Permissions:**
```bash
chmod 600 council_identity.key  # Owner read/write only
```

**Backup:**
```bash
# CRITICAL: Backup your key!
cp council_identity.key ~/safe_backup_location/
# Losing this key means losing your identity
```

### Rotation

To generate a new identity:
```bash
rm council_identity.key
# App will generate new key on next start
```

**Warning:** All previous signatures will still be valid but tied to old key!

### Import/Export

```rust
// Export for backup
let identity = SigningIdentity::load(path)?;
let bytes = identity.signing_key.to_bytes();
// Store bytes securely

// Import from backup
let signing_key = SigningKey::from_bytes(&bytes);
```

## Fingerprints

For human-readable identity display:

```rust
let fingerprint = public_key_fingerprint(&pubkey)?;
// Example: "A1B2C3D4E5F6G7H8"
```

**How it works:**
```
SHA256(public_key) → Take first 16 hex chars → Uppercase
```

**Display:**
```
Peer: A1B2C3D4E5F6G7H8
```

Users can verify they're talking to the same peer across sessions.

## Performance

**Ed25519 Benchmarks:**
- Key generation: ~50 μs
- Signing: ~50 μs
- Verification: ~150 μs

**Overhead:**
- Signature size: 64 bytes → 88 bytes Base64
- Public key size: 32 bytes → 44 bytes Base64
- Total: ~132 bytes per signed response

Negligible compared to typical AI response sizes (100s of KB).

## Comparison to PGP

| Feature | Ed25519 (Our Choice) | PGP/RSA |
|---------|---------------------|---------|
| Key size | 32 bytes | 256-512 bytes |
| Signature size | 64 bytes | 256+ bytes |
| Performance | Very fast | Slower |
| Security | 128-bit | Varies (2048-bit RSA ≈ 112-bit) |
| Complexity | Simple | Complex (multiple algorithms) |
| Use case | Digital signatures | Email encryption + signing |

**Why Ed25519?**
- Designed specifically for modern cryptography
- Faster and smaller than RSA
- No implementation vulnerabilities (unlike some RSA)
- Perfect for P2P networks (low overhead)
- Battle-tested (used by Signal, SSH, etc.)

## Testing

```bash
# Run crypto tests
cargo test crypto

# Specific tests
cargo test test_sign_and_verify
cargo test test_tamper_detection
cargo test test_wrong_signature
```

**Test Coverage:**
- ✅ Identity generation
- ✅ Sign and verify
- ✅ Tamper detection
- ✅ Wrong signature rejection
- ✅ Save/load keypair
- ✅ Fingerprint generation
- ✅ Deterministic signatures (timestamp-based)

## Future Enhancements

- [ ] Key rotation mechanism
- [ ] Multi-sig for consensus (require N of M signatures)
- [ ] Hardware security module (HSM) support
- [ ] Key revocation list
- [ ] Certificate authority (optional, for verified identities)
- [ ] Encrypted responses (Ed25519 + X25519 for ECDH)

## Security Best Practices

1. **Never share your private key!**
   - Keep `council_identity.key` secure
   - Set restrictive file permissions

2. **Backup your key**
   - Store encrypted backup offline
   - Losing key = losing identity

3. **Verify signatures**
   - Always check signatures before trusting responses
   - Reject unsigned responses in production

4. **Monitor for replay attacks**
   - Check timestamps
   - Reject old signatures (>5 min old)

5. **Use TLS for P2P transport**
   - Signatures prove message origin
   - TLS encrypts transport layer
   - Both are needed!

## Troubleshooting

**"Failed to load identity"**
→ Keypair file corrupted or wrong format. Delete and regenerate.

**"Signature verification failed"**
→ Response was tampered with OR wrong public key. Reject response.

**"Invalid public key encoding"**
→ Base64 decoding failed. Check format.

**Tests failing**
→ Check system time is correct (timestamps must match).

---

**Remember:** Cryptographic signatures are only part of security. Always combine with:
- TLS/SSL for transport encryption
- Reputation system for trust
- Rate limiting for DoS protection
- Input validation for all data
