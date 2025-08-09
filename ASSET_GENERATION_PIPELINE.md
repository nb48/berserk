# Asset Generation Pipeline

```mermaid
flowchart TB
  classDef prompt fill:#E8F0FE,stroke:#3B82F6,color:#1E3A8A;
  classDef script fill:#E8F5E9,stroke:#16A34A,color:#064E3B;
  classDef file fill:#FFF7ED,stroke:#F97316,color:#7C2D12;

  %% ===== PROMPT ORCHESTRATION =====
  subgraph PROMPT_ORCH["Prompt Orchestration"]
    direction TB
    B0["Brief for GPT"]:::prompt
    T1["Predefined Prompt Templates"]:::prompt
    FT1a["templates/1-silhouette-template.md"]:::file
    FT1b["templates/2-segmentation-template.md"]:::file
    FT1c["templates/3-joints-template.md"]:::file
    FT1d["templates/4-detail-masks-template.md"]:::file
    E1["GPT Expands Templates into Concrete Prompts"]:::prompt
    FE1a["prompts/1-silhouette-prompt.txt"]:::file
    FE1b["prompts/2-segmentation-prompt.txt"]:::file
    FE1c["prompts/3-joints-prompt.txt"]:::file
    FE1d["prompts/4-detail-masks-prompt.txt"]:::file
  end

  B0 --> E1
  T1 --> E1
  FT1a --> T1
  FT1b --> T1
  FT1c --> T1
  FT1d --> T1
  E1 --> FE1a
  E1 --> FE1b
  E1 --> FE1c
  E1 --> FE1d

  %% ===== PROMPT OUTPUTS =====
  subgraph PROMPTS["AI Prompt Outputs"]
    direction TB
    P1["1 Silhouettes (orthographic)"]:::prompt
    F1a["1-sil-front.png"]:::file
    F1b["1-sil-side.png"]:::file
    F1c["1-sil-top.png"]:::file

    P2["2 Segmentation Maps (color keyed)"]:::prompt
    F2a["2-seg-front.png"]:::file
    F2b["2-seg-side.png"]:::file
    F2c["2-seg-top.png"]:::file
    F2d["2-seg-color-key.json"]:::file

    P3["3 Joint Heatmaps"]:::prompt
    F3a["3-joints-front.png"]:::file
    F3b["3-joints-side.png"]:::file
    F3c["3-joints-top.png"]:::file

    P4["4 Tileable Surface Detail Masks"]:::prompt
    F4a["4-detail-grime-01.png"]:::file
    F4b["4-detail-crack-01.png"]:::file
  end

  FE1a --> P1
  FE1b --> P2
  FE1c --> P3
  FE1d --> P4

  %% ===== PROCEDURAL ASSET GENERATION =====
  subgraph PIPELINE["Procedural Asset Generation"]
    direction TB
    S5["5 View Align and Consistency Check"]:::script
    A5a["5a-aligned-sil-front.png"]:::file
    A5b["5a-aligned-sil-side.png"]:::file
    A5c["5a-aligned-sil-top.png"]:::file
    A5d["5b-aligned-seg-front.png"]:::file
    A5e["5b-aligned-seg-side.png"]:::file
    A5f["5b-aligned-seg-top.png"]:::file

    S6["6 Voxel Carve and Marching Cubes"]:::script
    A6a["6-raw-skull.obj"]:::file
    A6b["6-raw-jaw.obj"]:::file

    S7["7 Automated Remesh"]:::script
    A7a["7-clean-skull.obj"]:::file
    A7b["7-clean-jaw.obj"]:::file

    S8["8 Auto UV Unwrap"]:::script
    A8a["8-uv-skull.obj"]:::file
    A8b["8-uv-jaw.obj"]:::file

    S9["9 Bake Maps (curvature, AO, thickness)"]:::script
    A9a["9-curvature.png"]:::file
    A9b["9-ao.png"]:::file
    A9c["9-thickness.png"]:::file

    S10["10 Procedural PBR Synthesis"]:::script
    A10a["10-baseColor.png"]:::file
    A10b["10-normal.png"]:::file
    A10c["10-orm.png"]:::file

    S11["11 KTX2 Encode (UASTC with mips)"]:::script
    A11a["11-baseColor.ktx2"]:::file
    A11b["11-normal.ktx2"]:::file
    A11c["11-orm.ktx2"]:::file

    S12["12 Auto Rig from Heatmaps"]:::script
    A12a["12-rig.json"]:::file
    A12b["12-inverseBindMatrices.bin"]:::file

    S13["13 Build glTF and BIN"]:::script
    A13a["13.gltf"]:::file
    A13b["13.bin"]:::file

    S14["14 Pack GLB (embed KTX2)"]:::script
    A14a["skull.glb"]:::file

    S15["15 Validate"]:::script
    A15a["15-validate-report.json"]:::file
  end

  %% data flow
  F1a --> S5; F1b --> S5; F1c --> S5
  F2a --> S5; F2b --> S5; F2c --> S5
  S5 --> A5a; S5 --> A5b; S5 --> A5c; S5 --> A5d; S5 --> A5e; S5 --> A5f

  A5a --> S6; A5b --> S6; A5c --> S6
  S6 --> A6a; S6 --> A6b

  A6a --> S7; A6b --> S7
  S7 --> A7a; S7 --> A7b

  A7a --> S8; A7b --> S8; F2d --> S8
  S8 --> A8a; S8 --> A8b
  A8a --> S9; A8b --> S9
  S9 --> A9a; S9 --> A9b; S9 --> A9c
  A9a --> S10; A9b --> S10; A9c --> S10; F4a --> S10; F4b --> S10
  S10 --> A10a; S10 --> A10b; S10 --> A10c
  A10a --> S11; A10b --> S11; A10c --> S11
  S11 --> A11a; S11 --> A11b; S11 --> A11c

  A7a --> S12; A7b --> S12; F3a --> S12; F3b --> S12; F3c --> S12
  S12 --> A12a; S12 --> A12b

  A11a --> S13; A11b --> S13; A11c --> S13
  A12a --> S13; A12b --> S13
  S13 --> A13a; S13 --> A13b
  A13a --> S14; A13b --> S14
  S14 --> A14a
  A14a --> S15
  S15 --> A15a
  ```
