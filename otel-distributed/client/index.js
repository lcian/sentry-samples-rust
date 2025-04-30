import * as Sentry from '@sentry/node';

Sentry.init({
    dsn: "https://1507d323bbbe42429c7b97db31617dae@o447951.ingest.us.sentry.io/4509242707476480",
    tracesSampleRate: 1.0,
    debug: true,
    integrations: [
        new Sentry.Integrations.Http({ tracing: true })
    ],
});


async function main() {
    Sentry.startSpan({ name: "Fetch data", op: "custom" }, async () => {
        await fetch("https://example.com/1");
        await fetch("https://example.com/2");
        await fetch("https://example.com/3");
        const data = await fetch(`http://localhost:3001/hello`)
            .then(res => {
                if (!res.ok) {
                    const error = new Error(`HTTP error! status: ${res.status} ${res.statusText}`);
                    error.status = res.status;
                    Sentry.captureException(error);
                    console.error(`Request failed with status: ${res.status} ${res.statusText}`);
                    return null;
                }
                return res.json();
            })
            .catch(err => {
                console.error(`Error making request: `, err.message);
                Sentry.captureException(err);
                return null;
            });
        return data;
    });
}

main()
