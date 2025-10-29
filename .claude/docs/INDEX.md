# Orca Cost Calculator - Documentation Index

## 📚 Knowledge Base Structure

### Core Documentation
- [Main README](../../README_FINAL.md) - Quick start & overview
- [API Guide](API_GUIDE.md) - Web API integration guide
- [Architecture](ARCHITECTURE.md) - Technical architecture

### User Guides
- [Auto Cost Guide](../../AUTO_COST_GUIDE.md) - Complete CLI usage
- [Quick Start](../../QUICK_START.md) - 3-step tutorial
- [Samples Guide](../../SAMPLES.md) - Test models info

### Technical Docs
- [Volume Estimation](VOLUME_ESTIMATION.md) - How estimation works
- [Parser Details](PARSER.md) - G-code/3MF parsing
- [Materials Database](MATERIALS.md) - Filament properties

### Project Info
- [Project Summary](../../PROJECT_SUMMARY.md) - Complete overview
- [Final Summary](../../FINAL_SUMMARY.md) - What was delivered

## 🎯 Quick Links by Use Case

### For Users
- **Just want to calculate cost?** → [README_FINAL.md](../../README_FINAL.md)
- **Command line usage?** → [AUTO_COST_GUIDE.md](../../AUTO_COST_GUIDE.md)
- **Test models?** → [SAMPLES.md](../../SAMPLES.md)

### For Developers
- **API Integration?** → [API_GUIDE.md](API_GUIDE.md)
- **Understanding code?** → [ARCHITECTURE.md](ARCHITECTURE.md)
- **Extending?** → [PARSER.md](PARSER.md)

### For Web Deployment
- **Building web service?** → [API_GUIDE.md](API_GUIDE.md)
- **Security considerations?** → [SECURITY.md](SECURITY.md)
- **Rust vs Python?** → [TECH_STACK.md](TECH_STACK.md)

## 📁 File Organization

```
orca-calculate-cost/
├── README.md                    # Main entry point (NEW)
├── README_FINAL.md             # Detailed user guide
│
├── Core Tools:
├── auto_cost.py                # Main CLI tool
├── estimate_cost.py            # Volume estimator
├── orca_cost_calculator.py     # Parser library
│
├── .claude/docs/               # Knowledge base
│   ├── INDEX.md               # This file
│   ├── API_GUIDE.md           # API integration
│   ├── ARCHITECTURE.md        # Technical design
│   └── ...
│
├── Sample Models:
├── test_cube.stl
├── sample_*.stl
│
└── Legacy/Archive:
    └── smart_cost.py.OLD
```

## 🚀 Next Steps (Web Deployment)

1. **Read**: [TECH_STACK.md](TECH_STACK.md) - Rust vs Python analysis
2. **Read**: [API_GUIDE.md](API_GUIDE.md) - API design
3. **Read**: [SECURITY.md](SECURITY.md) - Security considerations

## 📝 Document Status

| Document | Status | Last Updated |
|----------|--------|--------------|
| INDEX.md | ✅ Current | 2025-10-29 |
| API_GUIDE.md | 🚧 Creating | 2025-10-29 |
| ARCHITECTURE.md | 🚧 Creating | 2025-10-29 |
| TECH_STACK.md | 🚧 Creating | 2025-10-29 |
| SECURITY.md | 🚧 Creating | 2025-10-29 |

---

**Maintained by**: Claude Code
**Project**: Orca Slicer Cost Calculator
**Version**: 2.0 (API-Ready)
