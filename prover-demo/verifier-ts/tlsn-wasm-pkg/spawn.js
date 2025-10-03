function registerMessageListener(target, type, callback) {
    const listener = async (event) => {
        const message = event.data;
        if (message && message.type === type) {
            await callback(message.data);
        }
    };

    target.addEventListener('message', listener);
}

// Register listener for the start spawner message.
registerMessageListener(self, 'web_spawn_start_spawner', async (data) => {
    console.log('🚀 spawn.js: start_spawner message received');
    const workerUrl = new URL(
        './spawn.js',
        import.meta.url
    );
    console.log('🚀 spawn.js: worker URL created:', workerUrl.toString());

    const [module, memory, spawnerPtr] = data;
    console.log('🚀 spawn.js: importing tlsn_wasm.js...');

    try {
        const pkg = await import('../../../tlsn_wasm.js');
        console.log('🚀 spawn.js: tlsn_wasm.js imported successfully');

        const exports = await pkg.default({ module, memory });
        console.log('🚀 spawn.js: WASM module initialized');

        const spawner = pkg.web_spawn_recover_spawner(spawnerPtr);
        console.log('🚀 spawn.js: spawner recovered');

        postMessage('web_spawn_spawner_ready');
        console.log('🚀 spawn.js: posted spawner_ready message');

        console.log('🚀 spawn.js: starting spawner.run()...');
        await spawner.run(workerUrl.toString());
        console.log('🚀 spawn.js: spawner.run() completed');

        exports.__wbindgen_thread_destroy();
        console.log('🚀 spawn.js: thread destroyed, closing worker');

        close();
    } catch (error) {
        console.error('❌ spawn.js: Error in start_spawner:', error);
        postMessage({ type: 'error', error: error.message });
        close();
    }
});

// Register listener for the start worker message.
registerMessageListener(self, 'web_spawn_start_worker', async (data) => {
    console.log('👷 spawn.js: start_worker message received');
    const [module, memory, workerPtr] = data;

    try {
        console.log('👷 spawn.js: importing tlsn_wasm.js for worker...');
        const pkg = await import('../../../tlsn_wasm.js');
        console.log('👷 spawn.js: tlsn_wasm.js imported for worker');

        const exports = await pkg.default({ module, memory });
        console.log('👷 spawn.js: WASM module initialized for worker');

        console.log('👷 spawn.js: starting worker...');
        pkg.web_spawn_start_worker(workerPtr);
        console.log('👷 spawn.js: worker started successfully');

        exports.__wbindgen_thread_destroy();
        console.log('👷 spawn.js: worker thread destroyed, closing');

        close();
    } catch (error) {
        console.error('❌ spawn.js: Error in start_worker:', error);
        postMessage({ type: 'error', error: error.message });
        close();
    }
});

/// Starts the spawner in a new worker.
export async function startSpawnerWorker(module, memory, spawner) {
    const startTime = Date.now();
    const workerId = Math.random().toString(36).substr(2, 9);

    console.log(`🏭 startSpawnerWorker[${workerId}] called at ${startTime}`);
    const workerUrl = new URL(
        './spawn.js',
        import.meta.url
    );
    console.log(`🏭 startSpawnerWorker[${workerId}] Worker URL:`, workerUrl.toString());

    try {
        const worker = new Worker(
            workerUrl,
            {
                name: `web-spawn-spawner-${workerId}`,
                type: 'module'
            }
        );

        worker.onerror = (error) => {
            console.error(`❌ startSpawnerWorker[${workerId}] worker error:`, error);
        };

        worker.onmessageerror = (error) => {
            console.error(`❌ startSpawnerWorker[${workerId}] message error:`, error);
        };

        console.log(`🏭 startSpawnerWorker[${workerId}] worker created successfully`);

        const data = [module, memory, spawner.intoRaw()];
        worker.postMessage({
            type: 'web_spawn_start_spawner',
            data: data
        })

        console.log(`🏭 startSpawnerWorker[${workerId}] message posted to worker`);

        // Add timeout to detect stuck workers
        const timeout = setTimeout(() => {
            console.error(`⏰ startSpawnerWorker[${workerId}] TIMEOUT after 30 seconds`);
            worker.terminate();
        }, 30000);

        await new Promise((resolve, reject) => {
            worker.addEventListener('message', function handler(event) {
                console.log(`🏭 startSpawnerWorker[${workerId}] received message:`, event.data);

                if (event.data === 'web_spawn_spawner_ready') {
                    clearTimeout(timeout);
                    worker.removeEventListener('message', handler);
                    const duration = Date.now() - startTime;
                    console.log(`✅ startSpawnerWorker[${workerId}] ready after ${duration}ms`);
                    resolve();
                } else if (event.data?.type === 'error') {
                    clearTimeout(timeout);
                    worker.removeEventListener('message', handler);
                    console.error(`❌ startSpawnerWorker[${workerId}] error:`, event.data.error);
                    reject(new Error(event.data.error));
                }
            });

            worker.addEventListener('error', function (error) {
                clearTimeout(timeout);
                console.error(`❌ startSpawnerWorker[${workerId}] worker error event:`, error);
                reject(error);
            });
        });

        const totalDuration = Date.now() - startTime;
        console.log(`🎉 startSpawnerWorker[${workerId}] completed successfully in ${totalDuration}ms`);
    } catch (error) {
        const duration = Date.now() - startTime;
        console.error(`💥 startSpawnerWorker[${workerId}] failed after ${duration}ms:`, error);
        throw error;
    }
}