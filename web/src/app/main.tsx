import { createRoot } from 'react-dom/client';
import { App } from './App';
import '../styles/studio.css';
import '../styles/playback.css';
import '../styles/studio-report.css';
const root = document.getElementById('root');
if (!root) throw new Error('Missing application root.');
createRoot(root).render(<App />);
