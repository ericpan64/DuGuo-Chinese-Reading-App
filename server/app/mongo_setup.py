import mongoengine

def global_init():
    """Run once, this starts mongoDB on default port 27017
    Local copy -- multi-server hosting requires more params (passed in **args)"""
    mongoengine.register_connection(alias='core', name='Zhongwen_DB')
    # needed to run in production (authentication)
    data = dict(
        username="from config or env",
        password="from config or env",
        host="server from config or env",
        port="port from config or env",
        authentication_source='admin',
        authentication_mechanism='SCRAM-SHA-1',
        ssl=True,
        ssl_cert_reqs='ssl.CERT_NONE'
    )

