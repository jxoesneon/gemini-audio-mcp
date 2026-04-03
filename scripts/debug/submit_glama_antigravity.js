const { chromium } = require('playwright');

(async () => {
  const browser = await chromium.connectOverCDP('http://127.0.0.1:9222');
  const context = browser.contexts()[0];
  const pages = context.pages();
  const page = pages.length > 0 ? pages[0] : await context.newPage();

  console.log('Navigating to Glama.ai submission page...');
  await page.goto('https://glama.ai/mcp/servers');
  await page.waitForLoadState('networkidle');

  const addServerButton = page.getByRole('button', { name: /Add Server/i });
  if (await addServerButton.isVisible()) {
    await addServerButton.click();
    console.log('Clicked Add Server button.');
    await page.waitForTimeout(2000);
    
    // Check if login modal is present and click GitHub login
    const githubLoginButton = page.locator('button:has(svg path[d*="M12 0c-6.626 0-12 5.373-12 12"])'); // Rough selector for GitHub icon
    // Or more simply, if there are multiple buttons in the modal, it's usually the second one
    const modalButtons = page.locator('div[role="dialog"] button');
    if (await modalButtons.count() > 0) {
        console.log('Modal detected. Clicking GitHub login...');
        // Glama's modal has Google, GitHub, Discord. GitHub is the second button.
        await modalButtons.nth(1).click();
        await page.waitForTimeout(5000); // Wait for potential OAuth redirect
        await page.screenshot({ path: 'glama_after_github_click.png' });
    }

    const repoUrl = 'https://github.com/jxoesneon/gemini-audio-mcp';
    const urlInput = page.getByPlaceholder(/github\.com/i).or(page.locator('input[type="url"]'));
    if (await urlInput.isVisible()) {
      await urlInput.fill(repoUrl);
      console.log(`Filled repo URL: ${repoUrl}`);
      await page.keyboard.press('Enter');
      console.log('Submitted form.');
      await page.waitForTimeout(5000);
      await page.screenshot({ path: 'glama_submission_success.png' });
    } else {
      console.log('Could not find URL input field.');
      await page.screenshot({ path: 'glama_no_input_final.png' });
    }
  }

  await browser.close();
})();
