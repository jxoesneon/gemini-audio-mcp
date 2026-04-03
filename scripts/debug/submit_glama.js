const { chromium } = require('playwright');

(async () => {
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage();
  
  console.log('Navigating to Glama.ai submission page...');
  await page.goto('https://glama.ai/mcp/servers');
  
  // Wait for the page to load
  await page.waitForLoadState('networkidle');
  
  // Look for a button or link to submit/add a server
  // Glama usually has an "Add Server" or similar button
  const submitButton = await page.getByRole('button', { name: /Add Server|Submit|Add/i }).first();
  if (await submitButton.isVisible()) {
    await submitButton.click();
    console.log('Clicked Add Server button.');
  }

  // Input the GitHub URL
  const repoUrl = 'https://github.com/jxoesneon/gemini-audio-mcp';
  // Find an input field for the URL
  const urlInput = await page.getByPlaceholder(/github\.com/i).or(page.locator('input[type="url"]')).first();
  if (await urlInput.isVisible()) {
    await urlInput.fill(repoUrl);
    console.log(`Filled repo URL: ${repoUrl}`);
    
    // Submit the form
    await page.keyboard.press('Enter');
    console.log('Submitted form.');
    
    // Wait for response or confirmation
    await page.waitForTimeout(5000);
    const screenshotPath = 'glama_submission.png';
    await page.screenshot({ path: screenshotPath });
    console.log(`Screenshot saved to ${screenshotPath}`);
  } else {
    console.log('Could not find URL input field. Taking screenshot for debug.');
    await page.screenshot({ path: 'glama_debug.png' });
  }

  await browser.close();
})();
