// Simple soldier model renderer
console.log('🎮 Starting Berserk...');

import { Engine } from '@babylonjs/core/Engines/engine';
import { Scene } from '@babylonjs/core/scene';
import { ArcRotateCamera } from '@babylonjs/core/Cameras/arcRotateCamera';
import { Vector3 } from '@babylonjs/core/Maths/math.vector';
import { Color3 } from '@babylonjs/core/Maths/math.color';
import { DirectionalLight } from '@babylonjs/core/Lights/directionalLight';
import { ImportMeshAsync } from '@babylonjs/core/Loading/sceneLoader';

// Import glTF loader
import '@babylonjs/loaders/glTF';

const BACKGROUND_COLOR = new Color3(0.02, 0.02, 0.025);

const canvas = document.getElementById('game') as HTMLCanvasElement;

if (!canvas) {
  console.error('❌ Canvas element not found!');
  throw new Error('Canvas not found');
}

// Create engine
const engine = new Engine(canvas, true);

const createScene = async () => {
  const scene = new Scene(engine);
  scene.clearColor = BACKGROUND_COLOR.toColor4(1);

  // Create camera with controls
  const camera = new ArcRotateCamera('camera', Math.PI * 1.25, Math.PI / 2.5, 6, new Vector3(0, 1, 0), scene);
  camera.attachControl(canvas, true);
  camera.lowerRadiusLimit = 3;
  camera.upperRadiusLimit = 12;

  // Create lighting
  const sun = new DirectionalLight("sun", new Vector3(-1, -2, -1), scene);
  sun.intensity = 2.2;

  // Load soldier model
  try {
    console.log('Loading soldier model...');
    const result = await ImportMeshAsync('/models/soldier.glb', scene);
    console.log('✅ Successfully loaded soldier model');

    // Position the model if needed
    if (result.meshes.length > 0) {
      const rootMesh = result.meshes[0];
      rootMesh.position.y = 0;
    }

  } catch (error) {
    console.error('Failed to load soldier model:', error);
  }

  return scene;
};

// Initialize everything
(async () => {
  try {
    console.log('Initializing scene...');
    const scene = await createScene();
    
    // Start render loop
    engine.runRenderLoop(() => {
      scene.render();
    });

    // Handle resize
    window.addEventListener('resize', () => {
      engine.resize();
    });

    console.log('✅ Berserk initialized successfully!');
    
  } catch (error) {
    console.error('❌ Failed to initialize:', error);
  }
})();
