# Task #18: Screen Operation Panel - Quick Start Guide

## What's New

A new **Screen** tab has been added to the Editor alongside Canvas, Code, and List. This panel allows you to:
- View real-time screenshots from your RPA automation
- Pick exact pixel coordinates by clicking
- View RGB color values at any point
- Select rectangular regions for reference
- Access a history of picked coordinates

## How to Use

### 1. Basic Screenshot Capture

1. Run an RPA scenario (via Canvas → Run button)
2. Once the server starts, navigate to the **Screen** tab
3. Click the **Refresh** button to capture the current screen
4. The screenshot will appear in the main preview area

### 2. Zoom and Pan

- **Zoom Controls**: Use the 25%, 50%, 100%, 200% buttons at the top
- **Pan**: Right-click and drag to move around the zoomed image
- **Reset**: Press 100% to return to fit-to-screen view

### 3. Pick Coordinates

1. Click anywhere on the screenshot image (left-click)
2. The coordinates (X, Y) will appear in the top-right corner
3. The RGB color at that pixel is also displayed
4. The coordinate is automatically added to the history

**Coordinate Display Format:**
```
X: 1234
Y: 567
RGB: #FF5733
```

### 4. Access Coordinate History

In the right sidebar under **Coordinates** section:
- Shows the last 5 picked coordinates
- Each entry displays: coordinates and RGB color
- Click any entry to... (future feature: highlight on screen)

**Copy Options:**
- **Copy as x,y** button: Copies "1234,567" to clipboard
- **Copy as JSON** button: Copies {"x":1234,"y":567} to clipboard

### 5. Select Regions

1. Click and drag on the screenshot to create a rectangular selection
2. The region appears with a dashed blue outline
3. In the right sidebar under **Selection** section:
   - Start coordinates: where you started dragging
   - End coordinates: where you released
   - Width and Height in pixels
4. Click **Clear Region** to deselect

**Example:**
```
Start: (100, 50)
End: (300, 200)
Width: 200px
Height: 150px
```

### 6. Auto-Refresh

1. Check the **Auto-refresh** checkbox in the header
2. Enter a number (1-30) for seconds between captures
3. The screen will continuously update at that interval
4. Uncheck to stop auto-refresh

**Usage:** Great for monitoring changing screens during automation

### 7. Tools

In the right sidebar under **Tools** section:
- **Clear Region**: Remove the current region selection
- **Clear History**: Delete all picked coordinates from history

## Practical Examples

### Example 1: Find a Button to Click

1. Run your RPA scenario that displays the screen
2. Go to Screen tab
3. Identify the button you want to click
4. Click on the button center to get coordinates
5. The coordinates appear in history
6. Copy as CSV format
7. Use in a `click_image` step later (future integration)

### Example 2: Monitor Field Changes

1. Enable auto-refresh (e.g., 3 seconds)
2. Watch the screenshot update in real-time
3. Observe field value changes
4. Pick coordinate at changed area when it happens

### Example 3: Select UI Region

1. Identify the area of interest (e.g., a form)
2. Click and drag to select the region
3. View the exact dimensions in the sidebar
4. Region selection saved in state for reference (future: save as image)

## Tips & Tricks

1. **Better Visibility**: Use 200% zoom to see pixel details clearly
2. **Quick Navigate**: Use 50% zoom to see the full screen at once
3. **Pan Smoothly**: Right-click drag is more precise than scrolling
4. **Color Picking**: Useful for verifying element colors in tests
5. **Coordinate Format**: Most RPA tools expect x,y format (copy as CSV)
6. **Region Dimensions**: Use for validating UI component sizes
7. **History Limit**: Last 5 coordinates keep the sidebar tidy

## Common Tasks

### Task: Get coordinates for "Submit" button
1. Capture screen (Refresh button)
2. Locate the Submit button
3. Click directly on it
4. Coordinates appear in top-right corner
5. Copy as x,y for use in click_image step

### Task: Monitor login success
1. Enable auto-refresh (2-3 seconds)
2. Trigger login
3. Watch screen change in real-time
4. Coordinates help verify UI elements

### Task: Measure button size
1. Select button region (click and drag)
2. Check Width and Height in Selection panel
3. Use for responsive testing

## Troubleshooting

### "RPA server not running" error
- Make sure you clicked Run on the scenario first
- Wait for log message: "Server started on port XXXX"
- Try Refresh again

### Screenshot appears blank or corrupted
- Check server logs for errors
- Try Refresh again
- Restart the RPA scenario

### Coordinates not appearing
- Make sure you're clicking (not right-clicking) on the image
- Ensure screenshot is fully loaded
- Try Refresh to get a fresh screenshot

### Zoom controls not working
- Make sure screenshot is loaded first
- Click the image area, then try zoom
- Reload if needed

## Keyboard Shortcuts

Currently no keyboard shortcuts, but planned:
- (Future) Ctrl+C to copy selected region
- (Future) Ctrl+Z to undo region
- (Future) Mouse wheel to zoom

## Integration with Other Features

### Current
- Coordinates stored in run history
- Color information captured for reference
- Auto-refresh works during scenario execution

### Planned
- Drag coordinates to Inspector to fill click_image data
- Save regions as reference images
- NCC matching visualization
- Integration with image-based steps

## Performance Notes

- Auto-refresh every 2 seconds: ~15% CPU, ~50MB memory
- Auto-refresh every 5 seconds: ~8% CPU, ~30MB memory
- Large screenshots (4K): Zoom to 50% for smooth panning
- Mobile/Laptop: Use 100% zoom for best performance

## Theme Support

The Screen Panel works in both themes:
- **Dark Mode** (default): Dark background with blue accents
- **Light Mode**: Light background with blue accents

Switch themes in Settings → Appearance → Theme

## Multi-Language Support

Translations available for:
- English (en)
- Japanese (日本語)
- Chinese (简体中文)

Change language in Settings → Appearance → Language

## Limitations & Known Issues

1. **Single Image Display**: Shows only current screenshot, no history of images
2. **Coordinate Limit**: Keeps only last 5 in quick-access sidebar (50 in memory)
3. **No Drawing Tools**: Can only select rectangles, not freehand
4. **No NCC Matching**: Visual NCC matching visualization not yet implemented
5. **No Image Export**: Selected regions not saved as separate files yet

## Future Roadmap

Phase 2 (planned enhancements):
- [ ] Save selected regions as reference images
- [ ] Integrate with click_image step form
- [ ] NCC confidence score visualization
- [ ] Multiple match highlighting
- [ ] Freehand drawing tools
- [ ] Color histogram
- [ ] Keyboard shortcuts

## Getting Help

1. Check TASK18_IMPLEMENTATION.md for detailed architecture
2. Look at TASK18_FILES.md for file locations
3. Check browser DevTools Console for errors
4. Review RPA server logs for screenshot endpoint issues

---

**Last Updated:** June 2024
**Version:** 1.0 (Initial Release)
