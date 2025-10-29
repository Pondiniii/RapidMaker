# Web API Design - Minimal & Fast

## 🎯 Goal

**User uploads STL → Gets instant price**

**UX Flow:**
1. User selects STL file
2. User selects material (dropdown: PLA, PETG, TPU, ABS, Nylon, ASA)
3. (Optional) System admin sets price per kg per material
4. Click "Calculate" → Instant result!

## 🚀 Minimal API Design

### Endpoint: POST /api/estimate

**Request:**
```json
{
  "stl_base64": "base64_encoded_stl_file",
  "material": "PLA",
  "infill": 20        // Optional, default: 20
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "volume_cm3": 8.00,
    "filament_g": 4.76,
    "cost": 0.12,
    "time_estimate": "2h 30m",
    "material": "PLA"
  }
}
```

**Error Response:**
```json
{
  "success": false,
  "error": "File too large (max 50MB)"
}
```

## 💰 Pricing Strategy

### Option 1: Admin-Configured (Recommended)
```python
# config.py
MATERIAL_PRICES = {
    "PLA": 24.99,    # per kg
    "PETG": 29.99,
    "ABS": 27.99,
    "TPU": 39.99,
    "Nylon": 44.99,
    "ASA": 32.99
}
```

**User sees**: Just material selection
**Backend**: Applies stored price

### Option 2: User-Provided
User can optionally override price (for custom filaments):
```json
{
  "material": "PLA",
  "cost_per_kg": 19.99  // Optional override
}
```

## 🎨 Frontend Example (Minimal)

```html
<form id="estimateForm">
  <input type="file" name="stl" accept=".stl" required>

  <select name="material" required>
    <option value="PLA">PLA - $24.99/kg</option>
    <option value="PETG">PETG - $29.99/kg</option>
    <option value="TPU">TPU - $39.99/kg</option>
  </select>

  <button type="submit">Calculate Cost</button>
</form>

<div id="result" style="display:none">
  <h3>Estimate:</h3>
  <p>Material: <span id="material"></span></p>
  <p>Filament: <span id="filament"></span>g</p>
  <p>Cost: $<span id="cost"></span></p>
  <p>Time: <span id="time"></span></p>
</div>

<script>
document.getElementById('estimateForm').onsubmit = async (e) => {
  e.preventDefault();

  const formData = new FormData(e.target);
  const file = formData.get('stl');
  const material = formData.get('material');

  // Read file as base64
  const reader = new FileReader();
  reader.onload = async () => {
    const base64 = reader.result.split(',')[1];

    const response = await fetch('/api/estimate', {
      method: 'POST',
      headers: {'Content-Type': 'application/json'},
      body: JSON.stringify({
        stl_base64: base64,
        material: material
      })
    });

    const data = await response.json();

    if (data.success) {
      document.getElementById('material').textContent = data.data.material;
      document.getElementById('filament').textContent = data.data.filament_g.toFixed(2);
      document.getElementById('cost').textContent = data.data.cost.toFixed(2);
      document.getElementById('time').textContent = data.data.time_estimate;
      document.getElementById('result').style.display = 'block';
    } else {
      alert('Error: ' + data.error);
    }
  };
  reader.readAsDataURL(file);
};
</script>
```

## 🔧 Backend Implementation (Flask)

```python
# api.py - MINIMAL VERSION
from flask import Flask, request, jsonify
from flask_cors import CORS
import base64
import tempfile
from pathlib import Path
from estimate_cost import VolumeBasedEstimator

app = Flask(__name__)
CORS(app)  # Enable CORS for web

# Material prices (admin configurable)
MATERIAL_PRICES = {
    "PLA": 24.99,
    "PETG": 29.99,
    "ABS": 27.99,
    "TPU": 39.99,
    "Nylon": 44.99,
    "ASA": 32.99
}

# File size limit (50MB)
MAX_FILE_SIZE = 50 * 1024 * 1024

@app.route('/api/estimate', methods=['POST'])
def estimate():
    try:
        data = request.get_json()

        # Validate input
        if 'stl_base64' not in data or 'material' not in data:
            return jsonify({
                'success': False,
                'error': 'Missing required fields'
            }), 400

        # Decode STL
        stl_data = base64.b64decode(data['stl_base64'])

        # Check file size
        if len(stl_data) > MAX_FILE_SIZE:
            return jsonify({
                'success': False,
                'error': 'File too large (max 50MB)'
            }), 400

        # Validate material
        material = data['material'].upper()
        if material not in MATERIAL_PRICES:
            return jsonify({
                'success': False,
                'error': 'Invalid material'
            }), 400

        # Save to temp file
        with tempfile.NamedTemporaryFile(suffix='.stl', delete=False) as tmp:
            tmp.write(stl_data)
            tmp_path = tmp.name

        try:
            # Estimate cost
            estimator = VolumeBasedEstimator()
            cost_per_kg = data.get('cost_per_kg', MATERIAL_PRICES[material])
            infill = data.get('infill', 20)

            result = estimator.estimate(
                tmp_path,
                material=material,
                cost_per_kg=cost_per_kg,
                infill=infill,
                supports=False
            )

            return jsonify({
                'success': True,
                'data': {
                    'volume_cm3': result['model_volume_cm3'],
                    'filament_g': result['filament_used_g'],
                    'cost': result['estimated_cost'],
                    'time_estimate': result['estimated_time'],
                    'material': result['material']
                }
            })

        finally:
            # Cleanup temp file
            Path(tmp_path).unlink()

    except Exception as e:
        return jsonify({
            'success': False,
            'error': str(e)
        }), 500

@app.route('/api/materials', methods=['GET'])
def get_materials():
    """Get available materials and prices"""
    return jsonify({
        'success': True,
        'materials': MATERIAL_PRICES
    })

if __name__ == '__main__':
    app.run(debug=True, host='0.0.0.0', port=5000)
```

## 🛡️ Security Features

### 1. File Validation
```python
def validate_stl(data):
    # Check file signature (STL starts with "solid" or binary header)
    if data[:5] == b'solid':
        return True  # ASCII STL
    elif len(data) > 84:
        return True  # Binary STL
    return False
```

### 2. Rate Limiting
```python
from flask_limiter import Limiter
from flask_limiter.util import get_remote_address

limiter = Limiter(
    app,
    key_func=get_remote_address,
    default_limits=["10 per minute", "100 per hour"]
)

@app.route('/api/estimate', methods=['POST'])
@limiter.limit("5 per minute")  # Stricter for uploads
def estimate():
    # ...
```

### 3. Timeout Protection
```python
import signal
from contextlib import contextmanager

@contextmanager
def timeout(seconds):
    def handler(signum, frame):
        raise TimeoutError("Processing took too long")
    signal.signal(signal.SIGALRM, handler)
    signal.alarm(seconds)
    try:
        yield
    finally:
        signal.alarm(0)

# Use in estimate:
with timeout(30):  # 30 second max
    result = estimator.estimate(...)
```

## 📦 Deployment (Docker)

```dockerfile
# Dockerfile
FROM python:3.9-slim

WORKDIR /app

# Install orca-slicer (for volume detection)
RUN apt-get update && apt-get install -y orca-slicer

COPY requirements.txt .
RUN pip install -r requirements.txt

COPY . .

# Run as non-root
RUN useradd -m appuser
USER appuser

EXPOSE 5000

CMD ["gunicorn", "--bind", "0.0.0.0:5000", "--timeout", "30", "--workers", "4", "api:app"]
```

```yaml
# docker-compose.yml
version: '3'
services:
  api:
    build: .
    ports:
      - "5000:5000"
    environment:
      - MAX_FILE_SIZE=52428800  # 50MB
    restart: unless-stopped
    deploy:
      resources:
        limits:
          memory: 512M
```

## 🚀 Quick Deploy

```bash
# 1. Create API wrapper
cp api.py orca-calculate-cost/

# 2. Install Flask
pip install flask flask-cors flask-limiter gunicorn

# 3. Run
python api.py

# 4. Test
curl -X POST http://localhost:5000/api/estimate \
  -H "Content-Type: application/json" \
  -d '{"stl_base64":"...","material":"PLA"}'
```

## 📊 User Experience

**Simple flow:**
1. User visits website
2. Drag & drop STL file
3. Select material from dropdown
4. Click "Calculate"
5. See instant result:
   - Filament needed: 4.76g
   - Estimated cost: $0.12
   - Print time: ~2h 30m

**No complicated options** (infill/supports/etc) - keep it simple!

## 🎯 Minimalist Approach

**Don't expose to user:**
- ❌ Infill percentage
- ❌ Support material
- ❌ Layer height
- ❌ Print speed

**Use defaults:**
- ✅ 20% infill (standard)
- ✅ No supports (user adds manually if needed)
- ✅ Admin-configured prices

**Result**: Clean, simple interface. User just picks material.

---

**Status**: ✅ Design complete
**Next**: Implement api.py
**Deploy**: Docker recommended
