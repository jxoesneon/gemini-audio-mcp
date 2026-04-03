const { spawn } = require('child_process');

const smithery = spawn('npx', ['-y', '@smithery/cli', 'mcp', 'publish', 'https://github.com/jxoesneon/gemini-audio-mcp', '-n', 'gemini-audio-mcp']);

smithery.stdout.on('data', (data) => {
    const output = data.toString();
    console.log(`STDOUT: ${output}`);
    
    if (output.includes('Please enter your Smithery API key')) {
        smithery.stdin.write('6ce671ff-280e-415e-8b49-583414df829f\n');
    }
    
    if (output.includes('doesn\'t exist yet. Create it?')) {
        smithery.stdin.write('y\n');
    }
});

smithery.stderr.on('data', (data) => {
    console.error(`STDERR: ${data}`);
});

smithery.on('close', (code) => {
    console.log(`Child process exited with code ${code}`);
});
