import re

file_path = '/home/francisco/Projects/Commanda/web/templates/index.html'
with open(file_path, 'r') as f:
    content = f.read()

icon_map = {
    "cloud-outline": "fa-solid fa-cloud",
    "home": "fa-solid fa-house",
    "settings-outline": "fa-solid fa-gear",
    "resize-outline": "fa-solid fa-expand",
    "contract-outline": "fa-solid fa-compress",
    "qr-code-outline": "fa-solid fa-qrcode",
    "musical-notes": "fa-solid fa-music",
    "play": "fa-solid fa-play",
    "pause": "fa-solid fa-pause",
    "play-skip-back": "fa-solid fa-backward-step",
    "play-skip-forward": "fa-solid fa-forward-step",
    "analytics-outline": "fa-solid fa-wind",
    "location-sharp": "fa-solid fa-location-dot",
}

def replace_ion_icon(match):
    # Match contains the full <ion-icon ...> tag
    tag = match.group(0)
    # Extract name
    name_match = re.search(r'name="([^"]+)"', tag)
    if not name_match:
        name_match = re.search(r"name='([^']+)'", tag)
    if not name_match:
        return tag # fallback
    
    name = name_match.group(1)
    fa_class = icon_map.get(name, "fa-solid fa-circle") # fallback
    
    # Extract class
    class_match = re.search(r'class="([^"]+)"', tag)
    if not class_match:
        class_match = re.search(r"class='([^']+)'", tag)
    
    extra_classes = class_match.group(1) if class_match else ""
    
    # Extract id
    id_match = re.search(r'id="([^"]+)"', tag)
    id_str = f' id="{id_match.group(1)}"' if id_match else ""
    
    return f'<i{id_str} class="{fa_class} {extra_classes}"></i>'

# Replace HTML tags
content = re.sub(r'<ion-icon\s+[^>]+></ion-icon>', replace_ion_icon, content)

# JS replacements
replacements = [
    ('weather_icon.name = conditionData.icon;', 'weather_icon.className = conditionData.icon + " text-4xl text-white/90";'),
    ('weather_icon_mobile.name = conditionData.icon;', 'weather_icon_mobile.className = conditionData.icon + " text-2xl text-white/90 shrink-0";'),
    ('music_status_icon.name = "pause";', 'music_status_icon.className = "fa-solid fa-pause text-[10px] text-white/70";'),
    ('music_status_icon.name = "play";', 'music_status_icon.className = "fa-solid fa-play text-[10px] text-white/70";'),
    ('music_status_icon_mobile.name = "pause";', 'music_status_icon_mobile.className = "fa-solid fa-pause text-[9px] text-white/70";'),
    ('music_status_icon_mobile.name = "play";', 'music_status_icon_mobile.className = "fa-solid fa-play text-[9px] text-white/70";')
]

for old, new in replacements:
    content = content.replace(old, new)

with open(file_path, 'w') as f:
    f.write(content)

print("Done")
