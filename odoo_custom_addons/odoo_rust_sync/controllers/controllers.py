# -*- coding: utf-8 -*-
# from odoo import http


# class OdooRustSync(http.Controller):
#     @http.route('/odoo_rust_sync/odoo_rust_sync', auth='public')
#     def index(self, **kw):
#         return "Hello, world"

#     @http.route('/odoo_rust_sync/odoo_rust_sync/objects', auth='public')
#     def list(self, **kw):
#         return http.request.render('odoo_rust_sync.listing', {
#             'root': '/odoo_rust_sync/odoo_rust_sync',
#             'objects': http.request.env['odoo_rust_sync.odoo_rust_sync'].search([]),
#         })

#     @http.route('/odoo_rust_sync/odoo_rust_sync/objects/<model("odoo_rust_sync.odoo_rust_sync"):obj>', auth='public')
#     def object(self, obj, **kw):
#         return http.request.render('odoo_rust_sync.object', {
#             'object': obj
#         })

