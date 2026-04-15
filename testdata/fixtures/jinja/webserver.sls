{% set service_name = "nginx" %}
webserver:
  pkg.installed:
    - name: {{ service_name }}
  service.running:
    - enable: true
