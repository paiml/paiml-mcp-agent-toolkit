#!/usr/bin/env bun

import { readFile, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import process from 'node:process';

async function fixBuildPedantic() {
    const buildPath = join(process.cwd(), 'server/build.rs');
    let content = await readFile(buildPath, 'utf-8');
    
    // Fix 1: Add #[allow(clippy::missing_panics_doc)] to verify_dependency_versions
    content = content.replace(
        'fn verify_dependency_versions() {',
        '#[allow(clippy::missing_panics_doc)]\nfn verify_dependency_versions() {'
    );
    
    // Fix 2: Change ureq::Error to &ureq::Error
    content = content.replace(
        'fn handle_download_failure(e: ureq::Error,',
        'fn handle_download_failure(e: &ureq::Error,'
    );
    
    // Fix 3: Convert match to let...else
    content = content.replace(
        `let input = match fs::read(path) {
        Ok(data) => data,
        Err(_) => return,
    };`,
        'let Ok(input) = fs::read(path) else { return };'
    );
    
    // Fix 4: Convert second match to let...else
    content = content.replace(
        `let compressed = match create_compressed_data(&input) {
        Some(data) => data,
        None => return,
    };`,
        'let Some(compressed) = create_compressed_data(&input) else { return };'
    );
    
    // Fix 5: Add #[allow(clippy::missing_panics_doc)] to compress_templates
    content = content.replace(
        'fn compress_templates() {',
        '#[allow(clippy::missing_panics_doc)]\nfn compress_templates() {'
    );
    
    // Fix 6: Add allow for cast_precision_loss
    content = content.replace(
        '(1.0 - total_compressed as f64 / total_original as f64) * 100.0',
        '#[allow(clippy::cast_precision_loss)]\n                (1.0 - total_compressed as f64 / total_original as f64) * 100.0'
    );
    
    // Fix 7: Add missing_errors_doc allow
    content = content.replace(
        'fn download_and_compress_assets() {',
        '#[allow(clippy::missing_errors_doc)]\nfn download_and_compress_assets() {'
    );
    
    // Save the file
    await writeFile(buildPath, content);
    console.log('Fixed build.rs pedantic issues');
}

fixBuildPedantic().catch(console.error);