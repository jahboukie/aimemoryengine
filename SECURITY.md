# 🔒 AI Memory Engine - Security Guide

## 🚨 CRITICAL: Secret Management

### **Environment Files (NEVER COMMIT!)**
The following files contain sensitive credentials and must NEVER be committed to git:

```
.env.keygen          # Keygen API credentials
.env                 # General environment variables
.env.*               # Any environment file variants
license.json         # User license storage
*.lic               # License files
```

### **Required Setup for Development**

1. **Create `.env.keygen` file** (locally only):
```bash
KEYGEN_ACCOUNT_ID=your_account_id_here
```

2. **Verify `.gitignore` protection**:
```bash
git check-ignore .env.keygen
# Should return: .env.keygen
```

## 🛡️ Security Features Implemented

### **License Security**
- ✅ **Node-locked licensing** prevents unauthorized usage
- ✅ **Machine fingerprinting** using SHA256 hashing
- ✅ **Cryptographic license validation** via Keygen API
- ✅ **Secure license storage** in user's home directory
- ✅ **No secrets in client code** - only license keys

### **API Security**
- ✅ **HTTPS-only communication** with Keygen API
- ✅ **License key authentication** for machine activation
- ✅ **Request timeout protection** (10 seconds)
- ✅ **Error handling** prevents information leakage

### **Data Protection**
- ✅ **Local license storage** in `~/.aimemoryengine/license.json`
- ✅ **No sensitive data logging** in production
- ✅ **Machine fingerprint anonymization** via SHA256

## 🔐 Production Deployment Checklist

### **Before Release:**
- [ ] Remove all debug logging
- [ ] Verify no secrets in codebase
- [ ] Test license validation edge cases
- [ ] Confirm `.gitignore` is comprehensive
- [ ] Validate error messages don't leak info

### **Environment Setup:**
- [ ] Production Keygen account configured
- [ ] License policies properly set
- [ ] Product tokens secured
- [ ] Backup authentication methods ready

## 🚨 Incident Response

### **If Secrets Are Accidentally Committed:**
1. **Immediately rotate** all exposed credentials
2. **Force push** to remove from git history
3. **Update** all affected systems
4. **Audit** for any unauthorized usage

### **License Breach Response:**
1. **Suspend** affected licenses immediately
2. **Investigate** usage patterns
3. **Update** security measures
4. **Notify** affected users if necessary

## 📞 Security Contacts

- **Primary:** Your security team
- **Keygen Support:** support@keygen.sh
- **Emergency:** Immediate credential rotation required

---
**Last Updated:** 2025-07-26
**Security Review:** Required before each release
