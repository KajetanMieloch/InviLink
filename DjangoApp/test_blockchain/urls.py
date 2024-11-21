from . import views
from django.urls import path

urlpatterns = [
    path('phantom_test/', views.phantom_test, name='phantom_test'),
    path('phantom_send/', views.phantom_send, name='phantom_send'),
    path('phantom_send_mobile/', views.phantom_send_mobile, name='phantom_send_mobile'),
    path('phantom_callback/', views.phantom_callback, name='phantom_callback'),
]