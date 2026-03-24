# Overall Benchmark Summary

**Date:** March 24, 2026  
**Libraries:** acadrust v0.3.1, dxf-rs v0.6, ACadSharp v3.4.9, ezdxf v1.4.3  
**Platform:** Windows, Release mode  
**Scales tested:** Small (100), Medium (1,000), Large (10,000), Huge (100,000 entities)  
**Iterations:** Small=100, Medium=50, Large=25, Huge=10

---

## Category Winners at a Glance

| Category | Fastest Library | Margin | Notes |
|---|---|---|---|
| **DXF Parse** | acadrust | 2–2.3× vs dxf-rs, 3–5× vs ACadSharp, 15–24× vs ezdxf | Clear leader at all scales |
| **DXF Write** | acadrust | 1.2–1.5× vs dxf-rs, 2–3× vs ACadSharp, 12–15× vs ezdxf | Consistent winner |
| **DXF Roundtrip** | acadrust | 2× vs dxf-rs, 3.5× vs ACadSharp, 24× vs ezdxf | Parse advantage dominates |
| **Binary DXF Parse** | acadrust ≈ dxf-rs | Neck and neck; ACadSharp 5–6×, ezdxf 23–27× slower | Rust libraries both strong |
| **Binary DXF Write** | acadrust | 3.8–5× vs dxf-rs, 8–13× vs ACadSharp, 66–112× vs ezdxf | acadrust dominates |
| **Binary DXF Roundtrip** | acadrust | 1.7× vs dxf-rs, 6.2× vs ACadSharp, 42× vs ezdxf | acadrust fastest at all scales |
| **DWG Parse** | acadrust | 3.4–5.4× vs ACadSharp | dxf-rs/ezdxf have no DWG support |
| **DWG Write** | acadrust | 2.1–3.9× vs ACadSharp | dxf-rs/ezdxf have no DWG support |
| **DWG Roundtrip** | acadrust | 2.9–3.3× vs ACadSharp | dxf-rs/ezdxf have no DWG support |

---

## Format Support Matrix

| Feature | dxf-rs | acadrust | ACadSharp | ezdxf |
|---|---|---|---|---|
| DXF Read | ✅ | ✅ | ✅ | ✅ |
| DXF Write | ✅ | ✅ | ✅ | ✅ |
| Binary DXF Read | ✅ | ✅ | ✅ | ✅ |
| Binary DXF Write | ✅ | ✅ | ✅ | ✅ |
| DWG Read | ❌ | ✅ | ✅ | ❌ |
| DWG Write | ❌ | ✅ | ✅ | ❌ |

---

## Key Takeaways

1. **acadrust is the overall fastest library**, winning or tying in every benchmark category.
2. **dxf-rs** is competitive for binary DXF parsing (matching acadrust) and is the second-fastest in most categories.
3. **ACadSharp** offers .NET ecosystem compatibility but is 3–13× behind acadrust. It supports binary DXF write.
4. **ezdxf** is the most widely-used Python DXF library but is 12–112× slower than acadrust, reflecting Python's interpreted nature.
5. The performance gap **widens at larger scales**, confirming acadrust's native Rust implementation scales most efficiently.
6. **DWG benchmarks** are exclusively acadrust vs ACadSharp (dxf-rs and ezdxf have no DWG support).
7. **v0.3.1 shows no performance regressions** compared to v0.3.0 — results are consistent across all categories.

---

## Detailed Reports

1. [DXF Parse](01_dxf_parse.md)
2. [DXF Write](02_dxf_write.md)
3. [DXF Roundtrip](03_dxf_roundtrip.md)
4. [Binary DXF Parse](04_binary_dxf_parse.md)
5. [Binary DXF Write](05_binary_dxf_write.md)
6. [Binary DXF Roundtrip](06_binary_dxf_roundtrip.md)
7. [DWG Parse & Write](07_dwg_parse_write.md)
8. [DWG Roundtrip](08_dwg_roundtrip.md)
