# CableThermal-DLR

![Build Status](https://github.com/byemir/cable-thermal-dlr/actions/workflows/ci.yml/badge.svg)
![License](https://img.shields.io/badge/license-AGPL--3.0-blue.svg)

**High-Precision Deterministic Thermal Rating Engine (Dynamic Line Rating & Transformer Limits)**

CableThermal-DLR is an enterprise-grade, real-time Dynamic Line Rating (DLR) and Power Transformer Thermal Overload SDK/CLI designed for Transmission/Distribution System Operators (TSOs/DSOs), renewable wind/solar farm interconnects, and heavy industrial substations. 

Based strictly on **IEEE 738, IEC 60287, and IEEE C57.91** standards, it leverages real-time weather and load telemetry to dynamically unlock 15-35% more transmission capacity safely.

## Features & Benchmarks

- **Zero-Allocation Core Engine**: Written in Rust (`#![no_std]` ready), generating C-ABI / Python bindings.
- **Microsecond Latency**: Solves complex thermal balance and state equations in **<200μs** per line segment.
- **Air-Gapped & Secure**: 100% offline edge-execution. 
- **Deterministic**: No probabilistic guesswork. Real physical heat-balance formulas.
- **Production-Ready**: Strongly typed physical bounds checking and `tracing` integrated telemetry logging.

## Quickstart (Developer Integration)

Integrating CableThermal-DLR into your SCADA or EMS system takes just 3 lines of code:

```rust
use cable_thermal_core::{Conductor, WeatherTelemetry, calculate_steady_state_ampacity};

// 1. Ingest telemetry & define asset (e.g., Drake ACSR)
let weather = WeatherTelemetry { ambient_temp: 35.0, wind_speed: 1.2, solar_irradiance: 950.0 };
let conductor = Conductor { diameter: 0.02814, resistance_per_m: 7.28e-5, emissivity: 0.5, solar_absorptivity: 0.5 };

// 2. Calculate real-time Ampacity limit (I_max) at 100°C boundary safely
match calculate_steady_state_ampacity(&conductor, &weather, 100.0) {
    Ok(i_max) => println!("Instantaneous Safe Ampacity: {:.2} A", i_max),
    Err(e) => eprintln!("Failed to calculate ampacity: {}", e),
}

// 3. (Enterprise) Predict transformer hot-spot & aging for emergency loading
// let state = cable_thermal_pro::transformer::calculate_transformer_overload(1.25, 35.0);
```

## Community Edition (Open Source) vs. Enterprise Pro

This project follows an **Open-Core / Dual-Licensing** model.

| Feature | Community Edition (AGPLv3) | Enterprise Pro |
|---------|---------------------------|----------------|
| **Target Audience** | Researchers, Hobbyists, Students | TSOs, DSOs, EPC Firms, Utilities |
| **Overhead Line (IEEE 738)** | Steady-State only | Transient / Dynamic Step Response |
| **Underground (IEC 60287)** | Single-core, static | Multi-core interaction matrices (IEC 60853) |
| **Transformer (IEEE C57.91)** | ❌ | Dynamic Hot-Spot & Loss-of-Life Tracking |
| **SCADA Modbus TCP Bridge** | ❌ | ✅ Built-in zero-allocation parser |
| **License Type** | AGPLv3 | Commercial Subscription |
| **License Validation** | None | Offline Ed25519 Cryptographic verification |

## Licensing & Monetization

**CableThermal-DLR Pro** is available for commercial deployment by utilities and engineering firms.
Our proprietary licensing operates 100% offline via Ed25519 signature verification, ensuring your critical SCADA infrastructure remains air-gapped while fully compliant.

👉 [**Purchase Enterprise License via Polar.sh**](https://polar.sh)

---
*Copyright © 2026 Emirhan CAMCI. All rights reserved.*
