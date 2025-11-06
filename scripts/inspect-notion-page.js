import { chromium } from 'playwright';

async function inspectNotionPage() {
  const browser = await chromium.launch({ headless: false });
  const context = await browser.newContext();
  const page = await context.newPage();

  try {
    console.log('Navigating to Notion page...');
    await page.goto('https://osh-web.notion.site/104-28-220-169-2a215617d76a8166a575e3ccdde83603', {
      waitUntil: 'networkidle',
      timeout: 60000
    });

    console.log('Waiting for page to load...');
    await page.waitForTimeout(3000);

    // Get the page title
    const title = await page.title();
    console.log('\n=== Page Title ===');
    console.log(title);

    // Get all headings
    console.log('\n=== Headings Found ===');
    const headings = await page.$$eval('[data-block-id]', blocks => {
      return blocks
        .filter(block => {
          const text = block.textContent?.trim();
          return text && (
            block.querySelector('[placeholder="Heading 1"]') ||
            block.querySelector('[placeholder="Heading 2"]') ||
            text.includes('Remote Monitoring') ||
            text.includes('Application Usage') ||
            text.includes('Terminal') ||
            text.includes('Screenshot')
          );
        })
        .map((block, idx) => ({
          index: idx,
          text: block.textContent?.trim().substring(0, 100),
          id: block.getAttribute('data-block-id')
        }));
    });
    console.log(JSON.stringify(headings, null, 2));

    // Look for tables
    console.log('\n=== Tables Found ===');
    const tables = await page.$$eval('table', tables => {
      return tables.map((table, idx) => ({
        index: idx,
        rows: table.querySelectorAll('tr').length,
        cols: table.querySelectorAll('th, td').length,
        preview: Array.from(table.querySelectorAll('tr')).slice(0, 2).map(tr =>
          Array.from(tr.querySelectorAll('th, td')).map(cell => cell.textContent?.trim())
        )
      }));
    });
    console.log(JSON.stringify(tables, null, 2));

    // Look for code blocks
    console.log('\n=== Code Blocks Found ===');
    const codeBlocks = await page.$$eval('pre, code', codes => {
      return codes.map((code, idx) => ({
        index: idx,
        type: code.tagName,
        preview: code.textContent?.trim().substring(0, 100)
      }));
    });
    console.log(JSON.stringify(codeBlocks, null, 2));

    // Get all block types in order
    console.log('\n=== Block Structure (All Blocks) ===');
    const allBlocks = await page.$$eval('[data-block-id]', blocks => {
      return blocks.slice(0, 30).map((block, idx) => {
        const text = block.textContent?.trim().substring(0, 60);
        const hasTable = block.querySelector('table') ? 'TABLE' : '';
        const hasCode = block.querySelector('pre, code') ? 'CODE' : '';
        return {
          index: idx,
          id: block.getAttribute('data-block-id'),
          preview: text,
          special: hasTable || hasCode || ''
        };
      });
    });
    console.log(JSON.stringify(allBlocks, null, 2));

    console.log('\n=== Inspection Complete ===');
    console.log('Browser will close in 10 seconds...');
    await page.waitForTimeout(10000);

  } catch (error) {
    console.error('Error inspecting page:', error.message);
  } finally {
    await browser.close();
  }
}

inspectNotionPage();
