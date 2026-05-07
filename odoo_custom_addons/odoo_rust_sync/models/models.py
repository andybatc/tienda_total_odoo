from odoo import models, fields, api
import requests
import logging

_logger = logging.getLogger(__name__)


class ProductTemplate(models.Model):
    _inherit = 'product.template'

    @api.model_create_multi
    def create(self, vals_list):
        """ Se ejecuta al crear nuevos productos """
        # 1. Llamar al super para crear los registros en la DB y obtener sus IDs
        records = super(ProductTemplate, self).create(vals_list)

        # 2. Notificar a Rust por cada registro creado
        for record in records:
            record._send_rust_webhook()

        return records

    def write(self, vals):
        # 1. Ejecutar el guardado normal de Odoo
        res = super(ProductTemplate, self).write(vals)

        # 2. Si cambian campos clave, notificamos a Rust
        # Evitamos campos técnicos para no saturar el webhook
        sync_fields = [
            'name',
            'list_price',
            'default_code',
            'standard_price'
        ]
        if any(f in vals for f in sync_fields):
            for record in self:
                record._send_rust_webhook()
        return res

    def _send_rust_webhook(self):
        """ Envía el ID al worker de Loco """
        url = "http://127.0.0.1:5150/api/webhooks/odoo/update"
        try:
            # Solo enviamos el ID; Rust se encarga de consultar la DB
            requests.post(
                url,
                json={
                    "odoo_id": self.id,
                },
                timeout=1
            )
        except Exception as e:
            _logger.error("Fallo de conexión con Rust Backend: %s", str(e))