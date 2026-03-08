import React, { useEffect, useState } from 'react';
import { createRoot } from 'react-dom/client';
import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { Layout } from './components/Layout';
import { globalStyles } from './theme';
import { Dashboard } from './pages/Dashboard';
import { Chat } from './pages/Chat';
import { Messages } from './pages/Messages';
import { Logs } from './pages/Logs';
import { CronJobs } from './pages/CronJobs';
import { Projects } from './pages/Projects';
import { ProjectDetail } from './pages/ProjectDetail';
import { TaskDetail } from './pages/TaskDetail';
import { DocumentDetail } from './pages/DocumentDetail';
import { LiveBoard } from './pages/LiveBoard';
import { Settings } from './pages/Settings';
import { Capabilities } from './pages/Capabilities';
import { Setup } from './pages/Setup';
import { Welcome } from './pages/Welcome';
import { getStatus } from './api';

function App(): React.ReactElement {
  const [backendOnline, setBackendOnline] = useState(false);

  useEffect(() => {
    const check = () => {
      getStatus()
        .then(() => setBackendOnline(true))
        .catch(() => setBackendOnline(false));
    };
    check();
    const interval = setInterval(check, 15000);
    return () => clearInterval(interval);
  }, []);

  return (
    <Layout backendOnline={backendOnline}>
      <Routes>
        <Route path="/" element={<Dashboard />} />
        <Route path="/dashboard" element={<Dashboard />} />
        <Route path="/chat" element={<Chat />} />
        <Route path="/chat/:conversationId" element={<Chat />} />
        <Route path="/messages" element={<Messages />} />
        <Route path="/messages/:chatId" element={<Messages />} />
        <Route path="/logs" element={<Logs />} />
        <Route path="/jobs" element={<CronJobs />} />
        <Route path="/projects" element={<Projects />} />
        <Route path="/projects/:id" element={<ProjectDetail />} />
        <Route path="/tasks/:id" element={<TaskDetail />} />
        <Route path="/documents/:id" element={<DocumentDetail />} />
        <Route path="/live-board" element={<LiveBoard />} />
        <Route path="/settings" element={<Settings />} />
        <Route path="/capabilities" element={<Capabilities />} />
        <Route path="/setup" element={<Setup />} />
        <Route path="/welcome" element={<Welcome />} />
      </Routes>
    </Layout>
  );
}

const styleEl = document.createElement('style');
styleEl.textContent = globalStyles;
document.head.appendChild(styleEl);

const root = document.getElementById('root');
if (root) {
  createRoot(root).render(
    <React.StrictMode>
      <BrowserRouter>
        <App />
      </BrowserRouter>
    </React.StrictMode>
  );
}
