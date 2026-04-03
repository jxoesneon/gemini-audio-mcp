#!/usr/bin/env node

const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');
const os = require('os');

/**
 * Gemini Audio MCP - NPM/NPX Wrapper
 * This script handles platform detection and binary execution for the Rust-based MCP server.
 */

function getBinaryName() {
    const platform = os.platform();
    const arch = os.arch();

    if (platform === 'darwin') {
        return arch === 'arm64' ? 'gemini-audio-mcp-macos-arm64' : 'gemini-audio-mcp-macos-x64';
    } else if (platform === 'win32') {
        return 'gemini-audio-mcp-windows-x64.exe';
    } else if (platform === 'linux') {
        return 'gemini-audio-mcp-linux-x64';
    }
    return null;
}

function run() {
    const binaryName = getBinaryName();
    if (!binaryName) {
        console.error(`Unsupported platform: ${os.platform()} ${os.arch()}`);
        process.exit(1);
    }

    // Prioritize local development build if it exists
    const localBuild = path.join(__dirname, 'target', 'release', 'gemini-audio-mcp');
    const localBuildWin = path.join(__dirname, 'target', 'release', 'gemini-audio-mcp.exe');
    
    let binaryPath = '';
    
    if (fs.existsSync(localBuild)) {
        binaryPath = localBuild;
    } else if (fs.existsSync(localBuildWin)) {
        binaryPath = localBuildWin;
    } else {
        // Look for pre-compiled binary in the package directory
        const packagedBinary = path.join(__dirname, 'bin', binaryName);
        if (fs.existsSync(packagedBinary)) {
            binaryPath = packagedBinary;
        } else {
            console.error('Binary not found. Please build the project with "cargo build --release" or install via a supported method.');
            process.exit(1);
        }
    }

    const child = spawn(binaryPath, process.argv.slice(2), {
        stdio: 'inherit',
        env: process.env
    });

    child.on('error', (err) => {
        console.error(`Failed to start binary: ${err.message}`);
        process.exit(1);
    });

    child.on('exit', (code) => {
        process.exit(code);
    });

    // Handle signals
    process.on('SIGINT', () => child.kill('SIGINT'));
    process.on('SIGTERM', () => child.kill('SIGTERM'));
}

if (require.main === module) {
    run();
}
