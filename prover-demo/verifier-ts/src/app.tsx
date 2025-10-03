import React, { ReactElement, useCallback, useState } from 'react';
import { createRoot } from 'react-dom/client';
import * as Comlink from 'comlink';
import { Watch } from 'react-loader-spinner';
import { Verifier as TVerifier } from 'tlsn-wasm';
import './app.scss';
import { HTTPParser } from 'http-parser-js';
import OverviewDiagram from './overview_prover_verifier.svg';

// Delay worker creation until needed to avoid HMR issues
let workerWrapper: any = null;

function getWorker() {
  if (!workerWrapper) {
    try {
      console.log('Creating worker...');
      const worker = new Worker(new URL('./worker.ts', import.meta.url));

      // Add error handlers
      worker.onerror = (error) => {
        console.error('Worker error:', error);
      };

      worker.onmessageerror = (error) => {
        console.error('Worker message error:', error);
      };

      console.log('Worker created successfully');
      workerWrapper = Comlink.wrap(worker);
    } catch (error) {
      console.error('Failed to create worker:', error);
      throw error;
    }
  }
  return workerWrapper;
}

const container = document.getElementById('root');
const root = createRoot(container!);

root.render(<App />);

// Simple console capture
let capturedLogs: string[] = [];
const originalLog = console.log;

// const serverUrl = 'https://swissbank.tlsnotary.org/balances';
const serverUrl = "https://raw.githubusercontent.com/tlsnotary/devconnect25_demo/refs/heads/dev/swissbank/src/data/swissbankdata.json"
// const websocketProxyUrl = `wss://notary.pse.dev/proxy`;
const proverProxyUrl = 'wss://localhost:8443/prove';

function App(): ReactElement {
  const [ready, setReady] = useState(false);
  const [processing, setProcessing] = useState(false);
  const [result, setResult] = useState<string | null>(null);
  const [consoleMessages, setConsoleMessages] = useState<string[]>([]);

  // Simple console capture
  React.useEffect(() => {
    console.log = (...args) => {
      const timestamp = new Date().toLocaleTimeString();
      const message = `[${timestamp}] ${args.join(' ')}`;
      capturedLogs.push(message);
      setConsoleMessages([...capturedLogs]);
      originalLog.apply(console, args);
    };

    return () => {
      console.log = originalLog;
    };
  }, []);

  // Initialize TLSNotary
  React.useEffect(() => {
    (async () => {
      const { init } = getWorker();

      // Calculate optimal concurrency: min(3, available cores - 1) to avoid hitting browser limits
      const maxConcurrency = Math.min(3, Math.max(1, (navigator.hardwareConcurrency || 4) - 1));

      await init({
        loggingLevel: 'Info',
        hardwareConcurrency: maxConcurrency
      });
      setReady(true);
      console.log(`🔧 TLSNotary initialized with ${maxConcurrency} threads`);
    })();
  }, []);

  const onClick = useCallback(async () => {
    setProcessing(true);
    capturedLogs = [];
    setConsoleMessages([]);
    console.log('Starting verifier demo...');

    let verifier: TVerifier;
    try {
      const { Verifier } = getWorker();

      console.log('Setting up Verifier');
      verifier = await new Verifier({
        max_sent_data: 2048,
        max_recv_data: 4096
      });
      console.log('Verifier class instantiated');
      await verifier.connect(proverProxyUrl);
      console.log('Connecting verifier to p2p proxy: done');
    } catch (e: any) {
      console.error('Error setting up verifier: ' + e.message);
      console.error('Error connecting verifier to p2p proxy: ' + e.message);
      setProcessing(false);
      return;
    }

    await new Promise((r) => setTimeout(r, 2000));

    console.log('Start verifier');
    // This needs to be called before we send the request
    // This starts the verifier and makes it wait for the prover to send the request
    const verified = verifier.verify();
    const result = await verified;
    console.log('Verification completed');

    const sent_b = result.transcript?.sent || [];
    const recv_b = result.transcript?.recv || [];

    const server_name = result.server_name

    console.log(`Verified server name: ${server_name}`);

    let recv = bytesToUtf8(recv_b);
    let sent = bytesToUtf8(sent_b);

    console.log('Verified data received');
    console.log(`Transcript sent: ${sent.substring(0, 100)}${sent.length > 100 ? '...' : ''}`);
    console.log(`Transcript received: ${recv.substring(0, 100)}${recv.length > 100 ? '...' : ''}`);

    console.log('Ready - verification completed successfully');

    setResult(`Sent to ${server_name}:\n` +
      sent +
      "\n" +
      `Received from ${server_name}:\n` +
      recv,
    );

    setProcessing(false);
  }, [setResult, setProcessing]);

  return (
    <div className="min-h-screen flex flex-col bg-slate-100">
      <div className="w-full p-4 bg-slate-800 text-white flex-shrink-0 shadow-md">
        <div className="flex items-center justify-between">
          <h1 className="text-xl font-bold">TLSNotary Devconnect Demo</h1>
          <a
            href="https://github.com/tlsnotary/devconnect25_demo"
            target="_blank"
            rel="noopener noreferrer"
            className="flex items-center space-x-2 text-gray-300 hover:text-white transition-colors duration-200 group"
            title="View source on GitHub"
          >
            <svg
              className="w-6 h-6 group-hover:scale-110 transition-transform duration-200"
              fill="currentColor"
              viewBox="0 0 24 24"
              xmlns="http://www.w3.org/2000/svg"
            >
              <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z" />
            </svg>
            <span className="text-sm font-medium">Source</span>
          </a>
        </div>
      </div>

      <div className="grid grid-cols-1 gap-4 p-4 flex-grow">
        <div className="flex flex-col bg-white rounded-lg shadow-md border border-gray-200 p-4">
          <p>TODO introduction</p>

          <div className="text-center text-gray-700 mb-6">
            {/* Architecture Overview Diagram */}
            <div className="my-6 flex justify-center">
              <div className="bg-gray-50 border border-gray-200 rounded-lg p-4">
                <h3 className="text-sm font-medium text-gray-700 mb-3 text-center">Demo Overview</h3>
                <OverviewDiagram />
              </div>
            </div>
          </div>

          <div className="text-center">
            <button
              onClick={!processing && ready ? onClick : undefined}
              disabled={processing || !ready}
              className={`
                inline-block px-6 py-3 rounded-xl font-semibold text-white mb-6 text-lg
                transition-all duration-200 ease-in-out transform
                shadow-lg hover:shadow-xl
                ${processing || !ready
                  ? 'bg-gray-400 cursor-not-allowed opacity-70'
                  : 'bg-gradient-to-r from-blue-600 to-blue-700 hover:from-blue-700 hover:to-blue-800 hover:scale-105 active:scale-95 hover:-translate-y-0.5'
                }
                focus:outline-none focus:ring-4 focus:ring-blue-300 focus:ring-opacity-50
                border-0 relative overflow-hidden
              `}
            >
              <span className="relative z-10 flex items-center justify-center space-x-2">
                {processing && (
                  <div className="w-5 h-5 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
                )}
                <span>{ready ? 'Verify Prover Server' : 'Initializing...'}</span>
              </span>
              {!processing && ready && (
                <div className="absolute inset-0 bg-gradient-to-r from-blue-400 to-blue-500 opacity-0 hover:opacity-20 transition-opacity duration-200"></div>
              )}
            </button>
          </div>

          {/* Console Log View */}
          <div className="mb-4">
            <h3 className="text-md font-semibold text-gray-800 mb-2">Console Log</h3>
            <div className="flex flex-col text-sm bg-slate-50 border border-slate-200 w-full h-48 py-2 overflow-y-auto rounded">
              {consoleMessages.map((m, index) => (
                <span
                  key={index}
                  data-testid="console-log"
                  className="px-3 py-1 text-slate-600 break-all"
                >
                  {m}
                </span>
              ))}
            </div>
          </div>

          <div className="mt-6 mb-4">
            {result && (
              <>
                <h3 className="text-md font-semibold text-gray-800 mb-2">Verified data:</h3>
                <>
                  <div className="bg-gray-100 border border-gray-300 p-4 rounded-lg mt-4">
                    <pre data-testid="proof-data" className="text-left text-sm text-gray-800 whitespace-pre-wrap overflow-auto">
                      {result}
                    </pre>
                  </div>
                </>

              </>
            )}

          </div>

          {/* POAP Link Section - Only shown when verification is successful */}
          {result && (
            <div className="mt-6 mb-4">
              <h3 className="text-md font-semibold text-gray-800 mb-2">🎉 Verification Successful!</h3>
              <div className="bg-green-50 border border-green-200 p-4 rounded-lg">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-green-800 font-medium mb-1">Claim your POAP!</p>
                    <p className="text-green-700 text-sm">
                      You've successfully verified data using TLSNotary.
                    </p>
                  </div>
                  <a
                    href="https://poap.gallery/event/your-event-id"
                    target="_blank"
                    rel="noopener noreferrer"
                    className="ml-4 px-4 py-2 bg-green-600 hover:bg-green-700 text-white font-medium rounded-lg transition-colors duration-200 flex items-center space-x-2"
                  >
                    <span>🏆</span>
                    <span>Claim POAP</span>
                  </a>
                </div>
              </div>
            </div>
          )}


        </div>
      </div>
    </div>
  );
}

function parseHttpMessage(buffer: Buffer, type: 'request' | 'response') {
  const parser = new HTTPParser(
    type === 'request' ? HTTPParser.REQUEST : HTTPParser.RESPONSE,
  );
  const body: Buffer[] = [];
  let complete = false;
  let headers: string[] = [];

  parser.onBody = (t) => {
    body.push(t);
  };

  parser.onHeadersComplete = (res) => {
    headers = res.headers;
  };

  parser.onMessageComplete = () => {
    complete = true;
  };

  parser.execute(buffer);
  parser.finish();

  if (!complete) throw new Error(`Could not parse ${type.toUpperCase()}`);

  return {
    info: buffer.toString('utf-8').split('\r\n')[0] + '\r\n',
    headers,
    body,
  };
}

function bytesToUtf8(array: number[]): string {
  return Buffer.from(array).toString("utf8").replaceAll('\u0000', '🙈');
}